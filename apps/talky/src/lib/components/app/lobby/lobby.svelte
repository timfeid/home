<script lang="ts">
	import type { Procedures } from '@feid/bindings';
	import { client, websocketClient } from '../../../client';
	import type { Unsubscribable } from '../../../rspc';
	import { onDestroy, onMount } from 'svelte';
	import { Client } from '@rspc/client';
	import { user } from '../../../user.svelte';
	import Button from '../../ui/button/button.svelte';

	let { joinCode }: { joinCode: string } = $props();
	let lobby: undefined | Procedures['lobby_subscribe']['output'] = $state(undefined);
	let unsubscribe: () => void | undefined;

	function onData(data: Procedures['lobby_subscribe']['output']) {
		console.log('got data', data);
		lobby = data;
	}

	async function getLobby(joinCode: string) {
		lobby = undefined;
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
		// console.log(response);
	}
</script>

{joinCode}

<Button onclick={doSomething}></Button>

{JSON.stringify(lobby)}
