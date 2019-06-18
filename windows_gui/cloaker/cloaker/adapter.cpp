
#include "stdafx.h"
#include "cloaker.h"
#include "adapter.h"
#include <fstream>

const WCHAR* FILE_EXTENSION = L".cloaker";
const INT32 FILE_SIGNATURE = 0xC10A4BED;

INT getMode(WCHAR *filename) {
	// if it ends with file extension, return decrypting
	if (endsWithExt(filename)) {
		return 1;
	}
	// open file, check first bytes for signatue. if present, return decrypt, else return encrypt
	std::fstream fs;
	fs.open(filename, std::fstream::in);
	UINT32 bytes = 0;
	fs >> bytes;
	if (bytes == FILE_SIGNATURE) {
		return 1;
	}
	return 0;
}

CString saveDialog(WCHAR *inFile, CHAR mode) {
	CString newName(inFile);
	if (mode == 0) { // encrypt, append extension
		newName += FILE_EXTENSION;
		CFileDialog dlg(FALSE, NULL, newName, OFN_OVERWRITEPROMPT);
		dlg.DoModal();
		return dlg.GetPathName();
	} else { // decrypt, chop off extension if there, otherwise prepend decrypted.
		if (endsWithExt(inFile)) {
			newName = newName.Left(newName.GetLength() - wcslen(FILE_EXTENSION));
		} else {
			newName = L"decrypted_" + newName;
		}
		CFileDialog dlg(FALSE, NULL, newName, OFN_OVERWRITEPROMPT);
		dlg.DoModal();
		return dlg.GetPathName();
	}
}

BOOL endsWithExt(WCHAR *s) {
	size_t len = wcslen(s);
	size_t extLen = wcslen(FILE_EXTENSION);
	return (len > extLen && (0 == _wcsicmp(FILE_EXTENSION, &s[len - extLen])));
}
