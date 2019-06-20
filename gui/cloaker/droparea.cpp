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
    QString filename, outFilename;
    QString password;
    QMessageBox msgBox;

    if (mimeData->hasUrls()) {
        QList<QUrl> urlList = mimeData->urls();
        if (urlList.size() > 1) {
            msgBox.setText("Only one file at a time can be decrypted");
            msgBox.exec();
            this->setBackgroundRole(QPalette::Dark);
            event->acceptProposedAction();
            return;
        }
        filename = urlList.at(0).toLocalFile();
        if (!QFileInfo(filename).isFile()) {
            msgBox.setText("Only single files can be processed. "
                "To encrypt a folder, please wrap it a .zip file or similar archive/compression format first.");
            msgBox.exec();
            this->setBackgroundRole(QPalette::Dark);
            event->acceptProposedAction();
            return;
        }
    } else {
        setText(tr("Only single files can be dropped"));
        msgBox.exec();
        this->setBackgroundRole(QPalette::Dark);
        event->acceptProposedAction();
        return;
    }

    Mode mode = getMode(filename);

    Outcome o;
    do {
        o = passwordPrompts(mode, &password);
        if (o == cancel) {
            this->setBackgroundRole(QPalette::Dark);
            event->acceptProposedAction();
            return;
        }
    } while (o);

    do {
        outFilename = saveDialog(filename, mode);
        if (outFilename == "") {
            // user hit cancel
            this->setBackgroundRole(QPalette::Dark);
            event->acceptProposedAction();
            return;
        } else if (QFileInfo(outFilename).exists()) {
            // warn and redo
            msgBox.setText("Must select filename that does not already exist.");
            msgBox.exec();
            o = redo;
        } else {
            o = success;
        }
    } while (o);

    setText("Working...");
    config = makeConfig(mode, password.toUtf8().data(), filename.toUtf8().data(), outFilename.toUtf8().data());
    if (config == nullptr) {
        msgBox.setText("Could not start transfer, possibly due to malformed password or filename.");
        msgBox.exec();
        this->setBackgroundRole(QPalette::Dark);
        event->acceptProposedAction();
        return;
    }
    ret_msg = start(config);
    msgBox.setText(ret_msg);
    msgBox.exec();
    destroyConfig(config);
    destroyCString(ret_msg);
    this->clear();
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
