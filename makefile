EXE = memu

run:
	rm -f $(EXE)
	gcc source/*.c `pkg-config --cflags --libs gtk+-3.0` -o $(EXE)
	./$(EXE)
