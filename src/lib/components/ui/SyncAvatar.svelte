<script lang="ts">
	import type { SyncUser } from '$lib/types';
	import { discordAvatarUrl } from '$lib/util';
	import Icon from '@iconify/svelte';
	import clsx from 'clsx';
	import type { ClassValue } from 'clsx';

	type Props = {
		user: SyncUser;
		class?: ClassValue;
	};

	let { user, class: classProp }: Props = $props();
</script>

{#if user.avatar}
	<img
		src={discordAvatarUrl(user.discordId, user.avatar)}
		class={[classProp, 'size-10 rounded-full shadow-lg']}
		alt=""
	/>
{:else}
	<Icon
		icon="mdi:account-circle"
		class={clsx(classProp, 'text-primary-500 block size-10 shadow-lg')}
	/>
{/if}
