#pragma once

// for functions rust lib references
#pragma comment(lib, "userenv.lib")
#pragma comment(lib, "ws2_32.lib")

// rust library function definitions
extern "C" void* makeConfig(CHAR, CHAR*, CHAR*, CHAR*);
extern "C" CHAR* start(void*);
extern "C" void destroyConfig(void*);
extern "C" void destroyCString(CHAR*);

// determines whether file is cloaked already
// returns 0 for encrypt and 1 for decrypt
INT getMode(WCHAR*);

// presents save dialog and returns output file path
CString saveDialog(WCHAR*, CHAR);

BOOL endsWithExt(WCHAR *s);
