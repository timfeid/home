<script lang="ts">
	import { page } from '$app/state';

	export let title: undefined | string = undefined;
	export let description: undefined | string = undefined;
	export let image: undefined | string = undefined;

	let metaProperties: any[] = [];
	let seoTitle = !title
		? 'timfeid.com'
		: title?.includes('timfeid')
			? title
			: `${title} - timfeid.com`;

	metaProperties = [
		{ charSet: 'utf-8' },
		{ name: 'description', content: description },
		{ property: 'og:site_name', content: 'Tim Feid' },
		{ property: 'og:title', content: seoTitle },
		{ property: 'og:url', content: `https://timfeid.com/${page.url.pathname}` },
		{ property: 'og:description', content: description },
		{ property: 'og:image', content: image },
		{ property: 'og:type', content: 'website' },
		{ name: 'twitter:title', content: seoTitle },
		{ name: 'twitter:card', content: 'summary_large_image' },
		{ name: 'twitter:image', content: image },
	].filter((prop) => !!prop.content);
</script>

<svelte:head>
	<title>{seoTitle}</title>

	{#each metaProperties as meta, index}
		<meta {...meta} />
	{/each}
</svelte:head>
