#pragma once


// CDropTarget dialog

class CDropTarget : public CDialogEx
{
	DECLARE_DYNAMIC(CDropTarget)

public:
	CDropTarget(CWnd* pParent = NULL);   // standard constructor
	virtual ~CDropTarget();

// Dialog Data
#ifdef AFX_DESIGN_TIME
	enum { IDD = IDD_DROPTARGET };
#endif

protected:
	virtual void DoDataExchange(CDataExchange* pDX);    // DDX/DDV support

	DECLARE_MESSAGE_MAP()
public:
	afx_msg void OnDropFiles(HDROP hDropInfo);
};
