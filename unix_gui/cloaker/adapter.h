#ifndef ADAPTER_H
#define ADAPTER_H

#endif // ADAPTER_H

#include <QFileDialog>
#include <QString>
#include <fstream>

// rust functions
extern "C" void *makeConfig(int, char*, char*, char*);
extern "C" char *start(void*);
extern "C" void destroyConfig(void*);
extern "C" void destroyCString(char*);

enum Mode {
    Encrypt = 0,
    Decrypt = 1,
};

static const char* FILE_EXTENSION = ".cloaker";
static const uint32_t FILE_SIGNATURE = 0xC10A4BED;

Mode getMode(QString filename);
QString saveDialog(QString inFile, Mode mode);
