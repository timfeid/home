<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import type { Snippet } from 'svelte';
	import SettingsSidebar from './settings-sidebar.svelte';
	import VoicePanel from './panels/voice-panel.svelte';
	import { User } from 'lucide-svelte';
	import Panel from './panels/panel.svelte';

	const menuItems = [
		{ id: 'voice', label: 'Voice & Video', icon: User, component: VoicePanel },
		{ id: 'z2voice', label: 'Voice & Video', icon: User, component: VoicePanel }
	];
	let { child }: { child: Snippet<[{ props: Record<string, unknown> }]> } = $props();
	let activePanel = $state<(typeof menuItems)[number]['id']>('voice');
	let ActivePanel = $derived(menuItems.find((item) => item.id === activePanel)?.component);
</script>

<Dialog.Root>
	<Dialog.Trigger {child} />
	<Dialog.Content class="h-screen w-screen max-w-full rounded-none p-0">
		<div class="flex">
			<SettingsSidebar {menuItems} bind:activePanel />
			{#if ActivePanel}
				<Panel>
					<ActivePanel />
				</Panel>
			{/if}
		</div>
	</Dialog.Content>
</Dialog.Root>
