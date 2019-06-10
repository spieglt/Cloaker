#ifndef ADAPTER_H
#define ADAPTER_H

#endif // ADAPTER_H

// rust functions
extern "C" void *makeConfig(int, char*, char*);
extern "C" char *start(void*);
extern "C" void destroyConfig(void*);
extern "C" void destroyCString(char*);
