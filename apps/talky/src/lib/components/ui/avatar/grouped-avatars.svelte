<script lang="ts">
	import * as Avatar from '$lib/components/ui/avatar/index';
	import type { UserResource } from '@talky/soundhouse';

	const sizeClasses = {
		xs: 'h-4 w-4 text-xs',
		sm: 'h-5 w-5 text-xs',
		md: 'h-8 w-8 text-sm',
		lg: 'h-10 w-10 text-base'
	};

	let {
		avatars,
		max = 5,
		size = 'sm'
	}: { avatars: UserResource[]; max?: number; size?: keyof typeof sizeClasses } = $props();

	let visibleAvatars = $derived(avatars.slice(0, max));
	let remaining = $derived(avatars.length - max);

	const colors = [
		'#FF80A0',
		'#C56792',
		'#33A086',
		'#A564D1',
		'#4D95FF',
		'#339BB5',
		'#FF6A40',
		'#7A8B94'
	];

	function getColor(firstName: string, lastName = '') {
		const initialF = firstName[0] || '';
		const initialL = lastName[0] || '';
		const seed = [initialF, initialL]
			.filter(Boolean)
			.map((c) => c.toLowerCase().charCodeAt(0) - 97)
			.reduce((a, b) => a + b, 0);

		return colors[seed % colors.length] || '#C7115A';
	}

	function getAvatarByName(options: {
		firstName: string;
		rounded?: boolean;
		color?: string;
		bold?: boolean;
		size?: number | string;
		background?: string;
		'font-size'?: string;
		format?: string;
	}) {
		const background = options.background || getColor(options.firstName).substring(1);
		const name = (options.firstName || '').substring(0, 1);

		const params = {
			background,
			name,
			color: options.color || 'fff',
			rounded: JSON.stringify(options.rounded || false),
			bold: JSON.stringify(options.bold || true),
			size: (options.size || 56).toString(),
			'font-size': options['font-size'] || '.6',
			format: options.format || 'svg'
		};

		return `https://ui-avatars.com/api/?` + new URLSearchParams(params).toString();
	}

	// Size mapping
</script>

<div class="flex -space-x-2">
	{#each visibleAvatars as avatar}
		<Avatar.Root class={sizeClasses[size]}>
			<Avatar.Image src={getAvatarByName({ firstName: avatar.user_id })} alt={avatar.user_id} />
			<Avatar.Fallback class="text-xs">{avatar.user_id.substring(0, 1)}</Avatar.Fallback>
		</Avatar.Root>
	{/each}

	{#if remaining > 0}
		<div
			class="{sizeClasses[
				size
			]} flex items-center justify-center rounded-full bg-muted text-muted-foreground"
		>
			<span class="text-xs font-medium">+{remaining}</span>
		</div>
	{/if}
</div>
