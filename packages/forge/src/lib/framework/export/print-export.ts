/**
 * Print export utility — renders styled HTML into a hidden iframe and triggers print.
 *
 * Uses an iframe instead of window.open() to avoid popup-blocker issues
 * when called from async command handlers.
 */

/** Print a styled view via a hidden iframe. */
export function exportViewToPrint(title: string, htmlContent: string): void {
  const rootStyles = getComputedStyle(document.documentElement);
  const fontFamily = rootStyles.getPropertyValue('--forge-font-family-ui').trim() || 'sans-serif';
  const monoFamily = rootStyles.getPropertyValue('--forge-font-family-mono').trim() || 'monospace';

  // Create a hidden iframe for printing
  const iframe = document.createElement('iframe');
  iframe.style.position = 'fixed';
  iframe.style.top = '-10000px';
  iframe.style.left = '-10000px';
  iframe.style.width = '800px';
  iframe.style.height = '600px';
  document.body.appendChild(iframe);

  const doc = iframe.contentDocument ?? iframe.contentWindow?.document;
  if (!doc) {
    console.warn('exportViewToPrint: cannot access iframe document');
    document.body.removeChild(iframe);
    return;
  }

  doc.open();
  doc.write(`<!DOCTYPE html>
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
  doc.close();

  // Wait for iframe content to render, then print and clean up
  iframe.onload = () => {
    iframe.contentWindow?.print();
    // Remove iframe after a short delay to let the print dialog appear
    setTimeout(() => {
      document.body.removeChild(iframe);
    }, 1000);
  };

  // Fallback: if onload doesn't fire (content already loaded synchronously)
  setTimeout(() => {
    if (iframe.parentNode) {
      iframe.contentWindow?.print();
      setTimeout(() => {
        if (iframe.parentNode) {
          document.body.removeChild(iframe);
        }
      }, 1000);
    }
  }, 500);
}

/** Escape HTML special characters for safe injection. */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
