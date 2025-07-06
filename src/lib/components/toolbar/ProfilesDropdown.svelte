<script lang="ts">
	import { activeProfile, profiles } from '$lib/stores.svelte';
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import clsx from 'clsx';
	import ProfilesDropdownItem from './ProfilesDropdownItem.svelte';
	import CreateProfileDialog from '$lib/components/dialogs/CreateProfileDialog.svelte';
	import { fade, fly } from 'svelte/transition';
	import { dropIn, dropOut } from '$lib/transitions';
	import DropdownArrow from '../ui/DropdownArrow.svelte';

	let open = $state(false);
	let newProfilePopupOpen = $state(false);
</script>

<DropdownMenu.Root bind:open>
	<DropdownMenu.Trigger
		class="group border-primary-600 text-primary-300 group-hover:text-primary-200 hover:bg-primary-800 flex min-w-40 shrink items-center border-r pr-4 pl-6"
	>
		<span class="mr-auto shrink truncate font-semibold">
			{$activeProfile?.name}
		</span>

		<div
			class="bg-primary-800 group-hover:bg-primary-700 mr-2 ml-6 rounded-sm px-2 py-0.5 text-sm font-medium"
		>
			{$activeProfile?.modCount}
		</div>

		<DropdownArrow {open} />
	</DropdownMenu.Trigger>
	<DropdownMenu.Content forceMount>
		{#snippet child({ wrapperProps, props, open })}
			<div {...wrapperProps}>
				{#if open}
					<div
						{...props}
						class="border-primary-600 bg-primary-800 z-30 flex max-h-[80lvh] min-w-40 flex-col gap-0.5 overflow-y-auto rounded-b-lg border p-1 shadow-lg"
						in:fly={dropIn}
						out:fade={dropOut}
					>
						{#each $profiles as profile, index}
							<ProfilesDropdownItem {profile} {index} />
						{/each}

						<DropdownMenu.Item
							class="bg-accent-700 hover:bg-accent-600 flex cursor-pointer items-center justify-center rounded-sm py-1 text-white"
							onclick={() => (newProfilePopupOpen = true)}
						>
							<Icon icon="mdi:plus" class="mr-1 text-lg" />
							New profile
						</DropdownMenu.Item>
					</div>
				{/if}
			</div>
		{/snippet}
	</DropdownMenu.Content>
</DropdownMenu.Root>

<CreateProfileDialog bind:open={newProfilePopupOpen} />
