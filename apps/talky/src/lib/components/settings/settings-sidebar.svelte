<script lang="ts">
	import type { Component } from 'svelte';

	let {
		activePanel = $bindable(),
		menuItems
	}: {
		activePanel: string;
		menuItems: { id: string; label: string; icon: Component }[];
	} = $props();
</script>

<div class="w-80 overflow-y-auto bg-sidebar">
	<div class="p-4">
		<h3 class="mb-2 text-xs font-semibold uppercase tracking-wide">User Settings</h3>
		<nav class="space-y-1">
			{#each menuItems as { id, label, icon: Icon }}
				<button
					onclick={() => (activePanel = id)}
					class={`flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-sm font-medium ${
						activePanel === id
							? 'bg-sidebar-accent  text-sidebar-accent-foreground'
							: ' hover:bg-sidebar-accent/55'
					}`}
				>
					<Icon class="size-5" />
					{label}
				</button>
			{/each}
		</nav>
	</div>
</div>
