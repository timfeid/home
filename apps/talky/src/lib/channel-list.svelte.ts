import { browser } from '$app/environment';
// import type { Procedures } from '@feid/bindings';
import { getContext } from 'svelte';
// import { client } from './client';
import { user } from './user.svelte';

// export type ListedChannel = Procedures['channel_list_in']['output'];

export class ChannelList {
	private currentUserId: undefined | string = $state(undefined);
	// channels: ListedChannel = $state([]);
	constructor() {
		$effect(() => {
			if (browser && user.user?.sub !== this.currentUserId) {
				this.refreshChannelList();
			}
		});
	}

	async refreshChannelList() {
		this.currentUserId = user.user?.sub;
		// if (this.currentUserId) {
		//   const response = await client.channel_list_in.query('');
		//   if (response.status === 'ok') {
		//     this.channels = response.data;
		//   }
		// }
	}

	cleanup() {}
}

export function withChannelList() {
	return getContext('channelList') as ChannelList;
}
