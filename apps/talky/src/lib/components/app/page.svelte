<script lang="ts">
	import type { Snippet } from 'svelte';
	import Seo from './seo.svelte';

	let {
		children,
		title,
		description,
		image,
	}: { children: Snippet; title?: string; description?: string; image?: string } = $props();
</script>

<Seo {title} {description} {image} />
{@render children()}
