#include "mainwindow.h"
#include "ui_mainwindow.h"

#include <QMessageBox>

MainWindow::MainWindow(QWidget *parent) :
    QMainWindow(parent),
    ui(new Ui::MainWindow)
{
    ui->setupUi(this);
}

MainWindow::~MainWindow()
{
    delete ui;
}

void MainWindow::on_actionAbout_Cloaker_triggered()
{
    QMessageBox::about(this, "About Cloaker",
                       "<h2>Cloaker v1.1</h2>"
                       "<p>Copyright (C) Theron Spiegl 2019</p>"
                       "<p>Licensed under the GNU General Public License v3.0</p>"
                       "<p>WARNING: if you encrypt a file and lose or forget the password, the file cannot be recovered.</p>"
    );
}
