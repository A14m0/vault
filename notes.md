# Binary Representation
There are two beginning parts to the file: there is the main payload of the 
file, which contains the actual file data, and there is the file table, which 
includes name, size, and offset information. It is also led by a magic byte 
sequence that will mean that the data has been successfully decrypted. 