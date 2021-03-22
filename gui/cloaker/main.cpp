#include "mainwindow.h"
#include <QApplication>

MainWindow *gMainWindow;

int main(int argc, char *argv[])
{
    QApplication a(argc, argv);
    MainWindow w;
    gMainWindow = &w;
    w.show();

    return a.exec();
}
