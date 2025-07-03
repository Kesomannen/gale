<script lang="ts">
	import { activeProfile, refreshProfiles, setActiveProfile } from '$lib/stores.svelte';
	import type { ProfileInfo } from '$lib/types';
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import clsx from 'clsx';
	import * as api from '$lib/api';
	import { pushInfoToast } from '$lib/toast';

	type Props = {
		index: number;
		profile: ProfileInfo;
	};

	let { index, profile }: Props = $props();

	let isActive = $derived(profile.id === $activeProfile?.id);

	async function deleteProfile() {
		let confirmed = confirm(`Are you sure you want to delete ${profile.name}?`);
		if (!confirmed) return;

		await api.profile.deleteProfile(index);

		pushInfoToast({
			message: `Deleted profile ${profile.name}.`
		});

		refreshProfiles();
	}
</script>

<DropdownMenu.Item
	class={[
		isActive
			? 'text-primary-300 hover:text-primary-200 font-medium'
			: 'text-primary-400 hover:text-primary-300',
		'group hover:bg-primary-700 flex cursor-default items-center rounded-md py-1 pr-1 pl-3 text-left'
	]}
	onclick={() => setActiveProfile(index)}
>
	{#if profile.sync !== null}
		<Icon icon="mdi:cloud" class="mr-2" />
	{/if}

	<span class="mr-3 grow">
		{profile.name}
	</span>

	<Icon icon="mdi:check" class={clsx(!isActive && 'invisible', 'text-accent-500 mx-2 text-lg')} />

	<div class="bg-primary-700 group-hover:bg-primary-600 mr-1 rounded-sm px-1.5 py-0.5 text-xs">
		{profile.modCount}
	</div>

	<button
		class="text-primary-400 rounded-sm p-1 hover:bg-red-600 hover:text-red-200"
		onclick={(evt) => {
			evt.stopPropagation();
			deleteProfile();
		}}
	>
		<Icon icon="mdi:delete" />
	</button>
</DropdownMenu.Item>
