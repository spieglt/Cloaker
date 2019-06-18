
#include "stdafx.h"
#include "cloaker.h"
#include <fstream>

const WCHAR* FILE_EXTENSION = L".cloaker";

INT getMode(WCHAR *filename) {
	size_t len = wcslen(filename);
	size_t extLen = wcslen(FILE_EXTENSION);
	// check file extension. if .cloaker, return decrypt
	if (len > extLen && (0 == _wcsicmp(FILE_EXTENSION, &filename[len - extLen]))) {
		return 1;
	}
	// open file, check first bytes for signatue. if present, return decrypt, else return encrypt
	std::fstream fs;
	fs.open(filename, std::fstream::in);
	UINT32 bytes = 0;
	fs >> bytes;
	if (bytes == 0xC10A4BE2) {
		return 1;
	}
	return 0;
}

CString saveDialog(WCHAR *inFile) {

	// output file dialog
	CFileDialog dlg(FALSE, L".cloaker", inFile, OFN_OVERWRITEPROMPT);
	dlg.DoModal();
	return dlg.GetPathName();

}
