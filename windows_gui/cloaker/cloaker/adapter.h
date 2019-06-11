#pragma once

// for functions rust lib references
#pragma comment(lib, "userenv.lib")
#pragma comment(lib, "ws2_32.lib")

// rust library function definitions
extern "C" void* makeConfig(CHAR, CHAR*, CHAR*);
extern "C" CHAR* start(void*);
extern "C" void destroyConfig(void*);
extern "C" void destroyCString(CHAR*);
