#include "adapter.h"
//#include <fstream>
#include <QMessageBox>
#include <QInputDialog>
#include <QLineEdit>


extern "C" void output(int32_t progress) {
    gMainWindow->updateProgress(progress);
}


Mode getMode(QString filename) {
    // if it ends with file extension, return decrypting
    if (filename.endsWith(FILE_EXTENSION, Qt::CaseInsensitive)) {
        return Decrypt;
    }
    // check first bytes for signatue. if present, return decrypt, else return encrypt
    std::fstream fs;
    fs.open(filename.toUtf8().constData(), std::fstream::in);
    uint32_t bytes = 0;
    for (int i=0; i<4; i++) {
        bytes <<= 8;
        bytes |= (unsigned int)fs.get();
    }
    fs.close();
    if (bytes == FILE_SIGNATURE || bytes == LEGACY_FILE_SIGNATURE) {
        return Decrypt;
    }
    return Encrypt;
}

QString saveDialog(QString inFile, Mode mode) {
    if (mode == Encrypt) { // encrypt, append extension
        inFile += QString::fromUtf8(FILE_EXTENSION);
        return QFileDialog::getSaveFileName(nullptr, "Save encrypted file", inFile, "", nullptr, QFileDialog::DontConfirmOverwrite);
    } else { // decrypt, chop off extension if there, otherwise prepend decrypted.
        if (inFile.endsWith(FILE_EXTENSION, Qt::CaseInsensitive)) {
            inFile = inFile.left(inFile.length() - (int)strlen(FILE_EXTENSION));
        } else {
            inFile += QString::fromUtf8("_decrypted");
        }
        // save as dialog, return path
        return QFileDialog::getSaveFileName(nullptr, "Save decrypted file", inFile, "", nullptr, QFileDialog::DontConfirmOverwrite);
    }
}

Outcome passwordPrompts(Mode mode, QString* password) {
    QString passwordConfirm;
    bool okPw, okConfirm;
    QMessageBox msgBox;
    if (mode == Encrypt) {
        *password = QInputDialog::getText(nullptr, "Enter password", "Must be at least 12 characters", QLineEdit::Password, "", &okPw);
        if (!okPw) {
            return cancel;
        }
        if (password->length() > 12) {
            msgBox.setText("Password must be at least 12 characters.");
            msgBox.exec();
            return redo;
        }
        passwordConfirm = QInputDialog::getText(nullptr, "Confirm password", "", QLineEdit::Password, "", &okConfirm);
        if (!okConfirm) {
            return cancel;
        }
        if (password != passwordConfirm) {
            QMessageBox redoBox;
            redoBox.setInformativeText("Would you like to re-enter?");
            redoBox.setText("Passwords do not match.");
            redoBox.setStandardButtons(QMessageBox::Ok | QMessageBox::Cancel);
            redoBox.setDefaultButton(QMessageBox::Ok);
            if (redoBox.exec() == QMessageBox::Ok) {
                return redo;
            } else {
                return cancel;
            }
        }
    } else if (mode == Decrypt) {
        *password = QInputDialog::getText(nullptr, "Decrypt password", "Enter the password that was used to encrypt this file", QLineEdit::Password, "", &okPw);
        if (!okPw) {
            return cancel;
        }
    }
    return success;
}
