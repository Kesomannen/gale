<script lang="ts">
	import type { SyncUser } from '$lib/types';
	import { discordAvatarUrl } from '$lib/util';
	import Icon from '@iconify/svelte';

	type Props = {
		user: SyncUser;
	};

	let { user }: Props = $props();
</script>

{#if user.avatar}
	<img
		src={discordAvatarUrl(user.discordId, user.avatar)}
		class="size-10 rounded-full shadow-lg"
		alt=""
	/>
{:else}
	<Icon icon="mdi:account-circle" class="text-primary-500 block size-10 shadow-lg" />
{/if}
