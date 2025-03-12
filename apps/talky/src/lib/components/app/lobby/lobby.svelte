<script lang="ts">
	import type { Procedures } from '@feid/bindings';
	import { client, websocketClient } from '../../../client';
	import type { Unsubscribable } from '../../../rspc';
	import { onDestroy, onMount } from 'svelte';
	import { Client } from '@rspc/client';
	import { user } from '../../../user.svelte';
	import Button from '../../ui/button/button.svelte';
	import Layout from './layout.svelte';
	import type { LobbyData, LobbyPresenceData } from './lobby';

	let { joinCode }: { joinCode: string } = $props();
	let presence: undefined | LobbyPresenceData = $state(undefined);
	let lobby: undefined | LobbyData = $state(undefined);
	let unsubscribe: () => void | undefined;

	function onData(data: LobbyPresenceData) {
		console.log('got data', data);
		presence = data;
	}

	async function getLobby(joinCode: string) {
		presence = undefined;
		if (unsubscribe) {
			unsubscribe();
		}
		try {
			if (websocketClient instanceof Client && user.accessToken) {
				unsubscribe = websocketClient.addSubscription(
					['lobby_subscribe', { join_code: joinCode, access_token: user.accessToken }],
					{
						onData,
					},
				);
			}
		} catch (e) {
			console.error(e);
		}
	}

	onMount(() => {
		getLobby(joinCode);
		doSomething();
	});

	onDestroy(() => {
		if (unsubscribe) {
			unsubscribe();
		}
	});

	$effect(() => {
		getLobby(joinCode);
	});

	async function doSomething() {
		const response = await client.lobby_join.mutate(joinCode);
		if (response.status !== 'ok') {
			return;
		}

		lobby = response.data;
		console.log(response);
	}
</script>

{joinCode}

<Button onclick={doSomething}></Button>

{JSON.stringify(presence)}

{#if lobby}
	<Layout {lobby} {presence}></Layout>
{/if}
