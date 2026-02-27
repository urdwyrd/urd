/**
 * SVG export utility — serialises an SVG element to a standalone .svg file.
 *
 * Resolves CSS custom properties to computed values so the exported SVG
 * renders correctly outside the application.
 */

/** Resolve CSS custom properties on an element tree to their computed values. */
function resolveCustomProperties(element: Element, computedStyles: CSSStyleDeclaration): void {
  // Process inline style if present
  if (element instanceof HTMLElement || element instanceof SVGElement) {
    const style = element.getAttribute('style');
    if (style && style.includes('var(--')) {
      const resolved = style.replace(/var\(--([^)]+)\)/g, (_match, prop) => {
        return computedStyles.getPropertyValue(`--${prop}`).trim() || _match;
      });
      element.setAttribute('style', resolved);
    }
  }

  // Recurse into children
  for (const child of element.children) {
    resolveCustomProperties(child, computedStyles);
  }
}

/** Export an SVG element as a downloadable .svg file. */
export function exportSvgFromElement(
  svgElement: SVGSVGElement,
  filename: string,
): void {
  // Clone the SVG to avoid modifying the live DOM
  const clone = svgElement.cloneNode(true) as SVGSVGElement;

  // Get computed styles from the document root for CSS custom property resolution
  const rootStyles = getComputedStyle(document.documentElement);

  // Resolve custom properties throughout the cloned tree
  resolveCustomProperties(clone, rootStyles);

  // Set explicit dimensions from the SVG's bounding box
  const bbox = svgElement.getBBox();
  const padding = 20;
  clone.setAttribute('width', String(bbox.width + bbox.x + padding));
  clone.setAttribute('height', String(bbox.height + bbox.y + padding));
  clone.setAttribute('xmlns', 'http://www.w3.org/2000/svg');

  // Inject a background rectangle matching the application background
  const bg = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
  bg.setAttribute('width', '100%');
  bg.setAttribute('height', '100%');
  bg.setAttribute('fill', rootStyles.getPropertyValue('--forge-bg-primary').trim() || '#1a1a2e');
  clone.insertBefore(bg, clone.firstChild);

  // Inject inline font styles
  const styleEl = document.createElementNS('http://www.w3.org/2000/svg', 'style');
  const fontFamily = rootStyles.getPropertyValue('--forge-font-family-ui').trim() || 'sans-serif';
  const textColour = rootStyles.getPropertyValue('--forge-text-primary').trim() || '#e0e0e0';
  styleEl.textContent = `text { font-family: ${fontFamily}; fill: ${textColour}; }`;
  clone.insertBefore(styleEl, clone.firstChild);

  // Serialise and download
  const serialiser = new XMLSerializer();
  const svgString = '<?xml version="1.0" encoding="UTF-8"?>\n' + serialiser.serializeToString(clone);

  const blob = new Blob([svgString], { type: 'image/svg+xml;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename.endsWith('.svg') ? filename : `${filename}.svg`;
  link.style.display = 'none';
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}
