<script lang="ts">
  import { onMount, type Snippet } from 'svelte';

  let {
    theme,
    defaultTheme,
    storageKey = 'theme',
    children,
  }: { theme?: string; defaultTheme?: string; storageKey?: string; children: Snippet } = $props();

  let themeValue = $state(defaultTheme);

  onMount(() => {
    // Get theme from local storage if available
    const storedTheme = localStorage.getItem(storageKey);
    if (storedTheme) {
      themeValue = storedTheme;
      applyTheme(themeValue);
    } else if (defaultTheme) {
      applyTheme(defaultTheme);
    }
  });

  // Watch for theme prop changes
  $effect(() => {
    if (theme !== undefined) {
      themeValue = theme;
      applyTheme(themeValue);
    }
  });

  function applyTheme(value: string) {
    document.documentElement.setAttribute('data-theme', value);
    localStorage.setItem(storageKey, value);
  }
</script>

{@render children()}
