#include "mainwindow.h"
#include "ui_mainwindow.h"

#include <QMessageBox>
#include <QProgressBar>

MainWindow::MainWindow(QWidget *parent) :
    QMainWindow(parent),
    ui(new Ui::MainWindow)
{
    ui->setupUi(this);
    ui->progBar->setVisible(false);
}

MainWindow::~MainWindow()
{
    delete ui;
}

void MainWindow::on_actionAbout_Cloaker_triggered()
{
    QMessageBox::about(this, "About Cloaker",
                       "<h2>Cloaker v4.0</h2>"
                       "<p>Copyright (C) Theron Spiegl 2021</p>"
                       "<p>Licensed under the GNU General Public License v3.0</p>"
                       "<p><a href=\"https://github.com/spieglt/cloaker\">https://spiegl.dev/cloaker</a></p>"
                       "<p><b>WARNING:</b> if you encrypt a file and lose or forget the password, the file cannot be recovered.</p>"
                       "<p><b>Backward compatibility notes:</b> "
                       "<p>If you are trying to decrypt a file made with version 1.0 or 1.1 of Cloaker (with Encrypt and Decrypt buttons), "
                       "the filename must end with the \".cloaker\" extension. Files encrypted with later versions are not subject to this restriction.</p>"
                       "<p>This version of Cloaker can decrypt files that were encrypted with previous versions, but previous versions cannot decrypt files encrypted with this version.</p>"
    );
}

void MainWindow::updateProgress(int percentage)
{
    if (!this->ui->progBar->isVisible()) {
        this->ui->progBar->setVisible(true);
    }
    this->ui->progBar->setValue(percentage);
}
