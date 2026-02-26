<script lang="ts">
  /**
   * GlobalMenuBar — top-level menu bar: File, Edit, View, Window, Help.
   */

  import MenuDropdown from './MenuDropdown.svelte';
  import type { MenuId } from './MenuRegistry';

  const MENUS: { id: MenuId; label: string }[] = [
    { id: 'file', label: 'File' },
    { id: 'edit', label: 'Edit' },
    { id: 'view', label: 'View' },
    { id: 'window', label: 'Window' },
    { id: 'help', label: 'Help' },
  ];

  let openMenu = $state<MenuId | null>(null);
  let anyOpen = $derived(openMenu !== null);

  function toggle(id: MenuId) {
    openMenu = openMenu === id ? null : id;
  }

  function close() {
    openMenu = null;
  }

  function hover(id: MenuId) {
    // Only switch on hover if a menu is already open
    if (anyOpen) {
      openMenu = id;
    }
  }
</script>

<div class="forge-menu-bar" role="menubar">
  <div class="forge-menu-bar__menus">
    {#each MENUS as menu}
      <MenuDropdown
        menuId={menu.id}
        label={menu.label}
        open={openMenu === menu.id}
        onToggle={() => toggle(menu.id)}
        onClose={close}
        onHover={() => hover(menu.id)}
      />
    {/each}
  </div>
</div>

<style>
  .forge-menu-bar {
    display: flex;
    align-items: center;
    height: 30px;
    padding: 0 var(--forge-space-sm);
    background: var(--forge-bg-primary);
    border-bottom: 1px solid var(--forge-border-zone);
    flex-shrink: 0;
  }

  .forge-menu-bar__menus {
    display: flex;
    align-items: center;
    gap: 0;
  }
</style>
