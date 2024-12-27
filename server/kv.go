package server

import (
	"bufio"
	"encoding/binary"
	"fmt"
	"io"
	"os"
	"sort"
	"sync/atomic"

	utils "github.com/johnietre/utils/go"
)

const (
	blockSize         = 12
	maxUint32  uint32 = 1<<32 - 1
	maxUpdates uint32 = 15
)

type Id = uint64

type inMemMap = map[Id]*utils.Mutex[uint32]

type MaxLogsStore struct {
	file                   *utils.Mutex[*os.File]
	fileName, tempFileName string

	//inMem map[Id]uint32
	inMem *utils.RWMutex[inMemMap]
	//inMemMtx sync.RWMutex
	updateCount atomic.Uint32

	// Do not assume values pointed to by mutex are 0
	numMtxPool *utils.Pool[*utils.Mutex[uint32]]

	minId, maxId Id
}

// TODO
func NewKVS() *MaxLogsStore {
	return &MaxLogsStore{
		file:         utils.NewMutex[*os.File](nil),
		fileName:     "",
		tempFileName: "",
		//inMem: make(map[Id]*utils.Mutex[uint32]),
		inMem: utils.NewRWMutex(make(inMemMap)),
		numMtxPool: utils.AlwaysNewPool(func() *utils.Mutex[uint32] {
			return utils.NewMutex(uint32(0))
		}),
	}
}

func (mls *MaxLogsStore) Add(id Id, num uint32) (_ uint32, err_ error) {
	return mls.Do(
		id,
		func(n uint32) (uint32, error) {
			if maxUint32-n < num {
				return n, ErrOverflow
			}
			return n + num, nil
		},
	)
}

func (mls *MaxLogsStore) Sub(id Id, num uint32) (_ uint32, err_ error) {
	return mls.Do(
		id,
		func(n uint32) (uint32, error) {
			if n < num {
				return n, ErrUnderflow
			}
			return n - num, nil
		},
	)
}

func (mls *MaxLogsStore) Set(id Id, num uint32) (err_ error) {
	_, err := mls.Do(id, func(_ uint32) (uint32, error) { return num, nil })
	return err
}

func (mls *MaxLogsStore) Do(
	id Id,
	f func(uint32) (uint32, error),
) (_ uint32, err_ error) {
	defer func() {
		if err_ == nil {
			err_ = mls.trySave()
		}
	}()

	/*
	  if n, err, ok := mls.tryDoInMem(id, f); ok {
	    return n, err
	  }
	*/

	defer mls.inMem.RUnlock()
	mtx, ok := (*mls.inMem.RLock())[id]
	if ok {
		nptr := mtx.Lock()
		defer mtx.Unlock()
		n := *nptr

		newN, err := f(n)
		if err != nil {
			return n, err
		}
		*nptr = newN
		mls.updated()
		return newN, nil
	}

	file := *mls.file.Lock()
	defer mls.file.Unlock()

	_, err := file.Seek(0, 0)
	if err != nil {
		return 0, &FileError{Err: err}
	}
}

/*
// Function f can cause other calls to wait if takes too long
func (mls *MaxLogsStore) tryDoInMem(
  id Id,
  f func(uint32) (uint32, error),
) (uint32, error, bool) {
  defer mls.inMem.RUnlock()
  mtx, ok := (*mls.inMem.RLock())[id]
  if !ok {
    return 0, nil, false
  }
  nptr := mtx.Lock()
  defer mtx.Unlock()
  n := *nptr

  newN, err := f(n)
  if err != nil {
    return n, err, true
  }
  *nptr = newN
  mls.updated()
  return newN, nil, true
}
*/

/*
func (mls *MaxLogsStore) getFile() (*os.File, error) {
}
*/

func (mls *MaxLogsStore) Save() error {
	return nil
}

func (mls *MaxLogsStore) trySave() (err_ error) {
	type inMemItem struct {
		Id  Id
		Num uint32
	}

	// Get the inMem map
	var inMem inMemMap
	if mls.updateCount.Load() < maxUpdates {
		if mptr, ok := mls.inMem.TryLock(); !ok {
			return nil
		} else {
			inMem = *mptr
		}
	} else {
		inMem = *mls.inMem.Lock()
	}
	defer mls.inMem.Unlock()
	if len(inMem) == 0 {
		return nil
	}

	// Get file and seek
	fileptr := mls.file.Lock()
	defer mls.file.Unlock()
	file := *fileptr

	_, err := file.Seek(0, 0)
	if err != nil {
		return &FileError{Err: err}
	}
	reader := bufio.NewReader(file)

	// Create temp file
	temp, err := os.Create(mls.tempFileName)
	if err != nil {
		return &TempFileError{Err: err}
	}
	defer temp.Close()
	writer := bufio.NewWriter(temp)

	// Copy over inMem data and sort on ID
	inMemArr := make([]inMemItem, 0, len(inMem))
	for id, num := range inMem {
		inMemArr = append(inMemArr, inMemItem{Id: id, Num: *num.Lock()})
		num.Unlock()
	}
	sort.Slice(inMemArr, func(i, j int) bool {
		return inMemArr[i].Id < inMemArr[j].Id
	})
	nextId := inMemArr[0].Id

	// Copy over, making changes when needed
	block := [blockSize]byte{}
	for {
		n, err := reader.Read(block[:])
		if err != nil {
			if err == io.EOF {
				break
			}
			return &FileError{Err: err}
		} else if n%blockSize != 0 {
			return &FileError{
				Err: fmt.Errorf(
					"expected multiple of %v bytes read, got %d",
					blockSize, n,
				),
			}
		}

		blockSlice := block[:n]
		for len(blockSlice) != 0 {
			id := binary.LittleEndian.Uint64(blockSlice[:8])
			// Add new IDs/nums
			for nextId != 0 && id > nextId {
				tblock := [blockSize]byte{}
				binary.LittleEndian.PutUint64(tblock[:8], inMemArr[0].Id)
				binary.LittleEndian.PutUint32(tblock[8:12], inMemArr[0].Num)
				if _, err := utils.WriteAll(writer, tblock[:]); err != nil {
					return &TempFileError{Err: err}
				}
				inMemArr = inMemArr[1:]
				if len(inMemArr) == 0 {
					nextId = 0
				} else {
					nextId = inMemArr[0].Id
				}
			}
			// Update num if necessasry
			if id == nextId {
				binary.LittleEndian.PutUint32(blockSlice[8:12], inMemArr[0].Num)
				inMemArr = inMemArr[1:]
				if len(inMemArr) == 0 {
					nextId = 0
				} else {
					nextId = inMemArr[0].Id
				}
			}
			// Write out
			if _, err := utils.WriteAll(writer, blockSlice[:blockSize]); err != nil {
				return &TempFileError{Err: err}
			}
			blockSlice = blockSlice[blockSize:]
		}
	}

	// Send rest of in mem to file
	for _, item := range inMemArr {
		binary.LittleEndian.PutUint64(block[:8], item.Id)
		binary.LittleEndian.PutUint32(block[8:12], item.Num)
		if _, err := utils.WriteAll(writer, block[:blockSize]); err != nil {
			return &TempFileError{Err: err}
		}
	}
	// Flush
	if err := writer.Flush(); err != nil {
		return &TempFileError{Err: err}
	}

	// Make temp file the main file
	temp.Close()
	file.Close()
	if err := os.Rename(mls.tempFileName, mls.fileName); err != nil {
		return &TempFileError{Err: err}
	}

	// Delete entries and return num mtxs to pool
	for id, num := range inMem {
		// TODO: shrink if larger than some size?
		delete(inMem, id)
		mls.numMtxPool.Put(num)
	}

	// Open the new file for future use
	f, err := os.Open(mls.fileName)
	if err != nil {
		return &FileError{Err: err}
	}
	*fileptr = f

	return nil
}

func (mls *MaxLogsStore) updated() {
	mls.updateCount.Add(1)
}

var (
	ErrOverflow  = fmt.Errorf("attempted addition would overflow")
	ErrUnderflow = fmt.Errorf("attempted subtraction would underflow")
	ErrNotExist  = fmt.Errorf("id does not exist")
)

type NotExistError struct {
	Id Id
}

func (nee *NotExistError) Error() string {
	return fmt.Sprintf("id %d does not exist", nee.Id)
}

type FileError struct {
	Err error
}

func (fe *FileError) Error() string {
	return fmt.Sprintf("file error: %v", fe.Err)
}

type TempFileError struct {
	Err error
}

func (tfe *TempFileError) Error() string {
	return fmt.Sprintf("temp file error: %v", tfe.Err)
}
