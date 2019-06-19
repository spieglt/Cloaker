#include "adapter.h"
//#include <fstream>


Mode getMode(QString filename) {
    // if it ends with file extension, return decrypting
    if (filename.endsWith(FILE_EXTENSION, Qt::CaseInsensitive)) {
        return Decrypt;
    }
    // open file, check first bytes for signatue. if present, return decrypt, else return encrypt
    std::fstream fs;
    fs.open(filename.toUtf8().constData(), std::fstream::in);
    uint32_t bytes;
    fs >> bytes;
    if (bytes == FILE_SIGNATURE) {
        return Decrypt;
    }
    return Encrypt;
}

QString saveDialog(QString inFile, Mode mode) {
    if (mode == Encrypt) { // encrypt, append extension
        inFile += QString::fromUtf8(FILE_EXTENSION);
        // save as dialog, return path
        return QFileDialog::getSaveFileName(nullptr, "Save encrypted file", inFile);
    } else { // decrypt, chop off extension if there, otherwise prepend decrypted.
        if (inFile.endsWith(FILE_EXTENSION, Qt::CaseInsensitive)) {
            inFile = inFile.left(inFile.length() - strlen(FILE_EXTENSION));
        } else {
            inFile += QString::fromUtf8("_decrypted");
        }
        // save as dialog, return path
        return QFileDialog::getSaveFileName(nullptr, "Save decrypted file", inFile);
    }
}
