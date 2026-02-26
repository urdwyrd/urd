import App from './App.svelte';
import { mount } from 'svelte';

const target = document.getElementById('app')!;
target.innerHTML = ''; // Clear the CSS-only splash screen

const app = mount(App, { target });

export default app;
