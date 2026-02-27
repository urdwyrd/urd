/**
 * Print export utility — opens a styled print window for browser/Tauri print-to-PDF.
 */

/** Open a print-ready window with the given HTML content. */
export function exportViewToPrint(title: string, htmlContent: string): void {
  const printWindow = window.open('', '_blank', 'width=800,height=600');
  if (!printWindow) {
    console.warn('exportViewToPrint: popup blocked — cannot open print window');
    return;
  }

  const rootStyles = getComputedStyle(document.documentElement);
  const fontFamily = rootStyles.getPropertyValue('--forge-font-family-ui').trim() || 'sans-serif';
  const monoFamily = rootStyles.getPropertyValue('--forge-font-family-mono').trim() || 'monospace';

  printWindow.document.write(`<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>${escapeHtml(title)}</title>
  <style>
    body {
      font-family: ${fontFamily};
      font-size: 12px;
      line-height: 1.4;
      margin: 20px;
      color: #1a1a1a;
      background: #fff;
    }
    h1 {
      font-size: 16px;
      margin-bottom: 12px;
      border-bottom: 1px solid #ccc;
      padding-bottom: 4px;
    }
    table {
      border-collapse: collapse;
      width: 100%;
    }
    th, td {
      border: 1px solid #ccc;
      padding: 4px 8px;
      text-align: left;
      font-size: 11px;
    }
    th {
      background: #f0f0f0;
      font-weight: 600;
    }
    code, pre {
      font-family: ${monoFamily};
      font-size: 11px;
    }
    @media print {
      body { margin: 0; }
    }
  </style>
</head>
<body>
  <h1>${escapeHtml(title)}</h1>
  ${htmlContent}
</body>
</html>`);

  printWindow.document.close();
  printWindow.focus();

  // Slight delay to allow rendering before triggering print
  setTimeout(() => {
    printWindow.print();
  }, 250);
}

/** Escape HTML special characters for safe injection. */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
