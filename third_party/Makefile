TARGET = flatbuffers.git

$(TARGET)/flatc: $(TARGET)
	(cd $(TARGET); cmake -DCMAKE_C_COMPILER=/usr/bin/gcc -DCMAKE_CXX_COMPILER=/usr/bin/g++ .)
	$(MAKE) -C $(TARGET) flatc

clean:
	$(MAKE) -C $(TARGET) clean
