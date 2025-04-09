<script lang="ts">
	import { ScrollArea } from '$lib/components/ui/scroll-area';

	let { children, footer = undefined } = $props();
</script>

<ScrollArea class="h-screen w-full">
	{@render children()}
</ScrollArea>
