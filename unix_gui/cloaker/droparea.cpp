/****************************************************************************
**
** Copyright (C) 2016 The Qt Company Ltd.
** Contact: https://www.qt.io/licensing/
**
** This file is part of the examples of the Qt Toolkit.
**
** $QT_BEGIN_LICENSE:BSD$
** Commercial License Usage
** Licensees holding valid commercial Qt licenses may use this file in
** accordance with the commercial license agreement provided with the
** Software or, alternatively, in accordance with the terms contained in
** a written agreement between you and The Qt Company. For licensing terms
** and conditions see https://www.qt.io/terms-conditions. For further
** information use the contact form at https://www.qt.io/contact-us.
**
** BSD License Usage
** Alternatively, you may use this file under the terms of the BSD license
** as follows:
**
** "Redistribution and use in source and binary forms, with or without
** modification, are permitted provided that the following conditions are
** met:
**   * Redistributions of source code must retain the above copyright
**     notice, this list of conditions and the following disclaimer.
**   * Redistributions in binary form must reproduce the above copyright
**     notice, this list of conditions and the following disclaimer in
**     the documentation and/or other materials provided with the
**     distribution.
**   * Neither the name of The Qt Company Ltd nor the names of its
**     contributors may be used to endorse or promote products derived
**     from this software without specific prior written permission.
**
**
** THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
** "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
** LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
** A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
** OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
** SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
** LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
** DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
** THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
** (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
** OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE."
**
** $QT_END_LICENSE$
**
****************************************************************************/

#include "adapter.h"
#include "droparea.h"

#include <QDragEnterEvent>
#include <QFileInfo>
#include <QInputDialog>
#include <QLineEdit>
#include <QMessageBox>
#include <QMimeData>

DropArea::DropArea(QWidget *parent)
    : QLabel(parent)
{
    setBackgroundRole(QPalette::Dark);
    this->encrypting = true;
}

void DropArea::dragEnterEvent(QDragEnterEvent *event)
{
    setBackgroundRole(QPalette::Highlight);
    event->acceptProposedAction();
}

void DropArea::dragMoveEvent(QDragMoveEvent *event)
{
    event->acceptProposedAction();
}

void DropArea::dropEvent(QDropEvent *event)
{
    const QMimeData *mimeData = event->mimeData();
    QString filename, password, passwordConfirm;
    bool okPw, okConfirm;
    int mode = this->encrypting? 0 : 1;
    void *config = nullptr;
    QString ret_msg;
    QMessageBox msgBox;

    if (mimeData->hasUrls()) {
        QList<QUrl> urlList = mimeData->urls();
        if (urlList.size() > 1) {
            if (this->encrypting) {
                msgBox.setText("To avoid leaving unencrypted partial files in case of program failure, only one file can be encrypted at a time. \
                    To encrypt multiple files, please wrap them in a .zip file or similar archive/compression format first.");
            } else {
                msgBox.setText("Only one file at a time can be decrypted");
            }
            msgBox.exec();
            goto CleanUp;
        }
        filename = urlList.at(0).path();
        if (!QFileInfo(filename).isFile()) {
            msgBox.setText("Only single files can be processed. \
                To encrypt a folder, please wrap it a .zip file or similar archive/compression format first.");
            msgBox.exec();
            goto CleanUp;
        }
    } else {
        setText(tr("Only single files can be dropped"));
        msgBox.exec();
        goto CleanUp;
    }

PasswordPrompts:
    password = QInputDialog::getText(this, "Enter password", "", QLineEdit::Password, "", &okPw);
    if (!okPw) {
        goto CleanUp;
    }
    if (this->encrypting) {
        passwordConfirm = QInputDialog::getText(this, "Confirm password", "", QLineEdit::Password, "", &okConfirm);
        if (!okConfirm) {
            goto CleanUp;
        }
        if (password != passwordConfirm) {
            QMessageBox redoBox;
            redoBox.setInformativeText("Would you like to re-enter?");
            redoBox.setText("Passwords do not match.");
            redoBox.setStandardButtons(QMessageBox::Ok | QMessageBox::Cancel);
            redoBox.setDefaultButton(QMessageBox::Ok);
            if (redoBox.exec() == QMessageBox::Ok) {
                goto PasswordPrompts;
            } else {
                goto CleanUp;
            }
        }
    }

    setText("Working...");
    config = makeConfig(mode, password.toUtf8().data(), filename.toUtf8().data());
    if (config == nullptr) {
        msgBox.setText("Could not start transfer, possibly due to malformed password or filename.");
        msgBox.exec();
        goto CleanUp;
    }

    msgBox.setText(start(config));
    msgBox.exec();
    this->clear();

CleanUp:
    setBackgroundRole(QPalette::Dark);
    event->acceptProposedAction();
}

void DropArea::dragLeaveEvent(QDragLeaveEvent *event)
{
    clear();
    event->accept();
}

void DropArea::clear()
{
    setText(tr("1. Select mode above\n\n\n2. Drop files here"));
    setBackgroundRole(QPalette::Dark);
}

void DropArea::setEncrypt()
{
    this->encrypting = true;
}

void DropArea::setDecrypt()
{
    this->encrypting = false;
}
