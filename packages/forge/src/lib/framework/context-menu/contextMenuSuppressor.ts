/**
 * Global right-click suppression + routing to custom context menus.
 */

let onContextMenuRequest: ((e: MouseEvent) => void) | null = null;

export function installContextMenuSuppressor(handler: (e: MouseEvent) => void): () => void {
  onContextMenuRequest = handler;

  const listener = (e: MouseEvent) => {
    e.preventDefault();
    handler(e);
  };

  document.addEventListener('contextmenu', listener);

  return () => {
    document.removeEventListener('contextmenu', listener);
    onContextMenuRequest = null;
  };
}
