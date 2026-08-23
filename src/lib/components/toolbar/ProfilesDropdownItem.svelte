<script lang="ts">
	import type { ProfileInfo } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import clsx from 'clsx';
	import * as api from '$lib/api';
	import { pushInfoToast } from '$lib/toast';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import profiles from '$lib/state/profile.svelte';

	type Props = {
		index: number;
		profile: ProfileInfo;
	};

	let { index, profile }: Props = $props();

	let isActive = $derived(profile.id === profiles.active?.id);

	async function deleteProfile() {
		let confirmed = await confirm(`Are you sure you want to delete ${profile.name}?`);
		if (!confirmed) return;

		await api.profile.deleteProfile(profile.id);

		pushInfoToast({
			message: `Deleted profile ${profile.name}.`
		});
	}
</script>

<DropdownMenu.Item
	class={[
		isActive
			? 'text-primary-700 hover:text-primary-800 dark:text-primary-300 dark:hover:text-primary-200 font-medium'
			: 'text-primary-500 hover:text-primary-600 dark:text-primary-400 dark:hover:text-primary-300',
		'group dark:hover:bg-primary-700 hover:bg-primary-200 flex cursor-default items-center rounded py-1 pr-1 pl-3 text-left'
	]}
	onclick={() => profiles.setActive(index)}
>
	{#if profile.sync !== null}
		<Icon icon="mdi:cloud" class="mr-2" />
	{/if}

	<span class="mr-3 grow">
		{profile.name}
	</span>

	<Icon
		icon="mdi:check"
		class={clsx(!isActive && 'invisible', 'text-accent-600 dark:text-accent-500 mx-2 text-lg')}
	/>

	<div
		class="dark:bg-primary-700 dark:group-hover:bg-primary-600 bg-primary-200 group-hover:bg-primary-300 mr-1 rounded-sm px-1.5 py-0.5 text-xs"
	>
		{profile.modCount}
	</div>

	<IconButton
		label="Delete profile"
		icon="mdi:delete"
		color="red"
		onclick={(evt) => {
			evt.preventDefault();
			evt.stopPropagation();
			deleteProfile();
		}}
	/>
</DropdownMenu.Item>
