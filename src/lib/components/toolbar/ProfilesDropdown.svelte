<script lang="ts">
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import ProfilesDropdownItem from './ProfilesDropdownItem.svelte';
	import CreateProfileDialog from '$lib/components/dialogs/CreateProfileDialog.svelte';
	import { fade, fly } from 'svelte/transition';
	import { dropIn, dropOut } from '$lib/transitions';
	import DropdownArrow from '../ui/DropdownArrow.svelte';
	import profiles from '$lib/state/profile.svelte';

	let open = $state(false);
	let createDialogOpen = $state(false);
</script>

<DropdownMenu.Root bind:open>
	<DropdownMenu.Trigger
		class="group border-primary-600 text-primary-300 group-hover:text-primary-200 hover:bg-primary-800 flex min-w-40 shrink items-center border-r pr-4 pl-6"
	>
		<span class="mr-auto shrink truncate font-semibold">
			{profiles.active?.name}
		</span>

		<div
			class="bg-primary-800 group-hover:bg-primary-700 mr-2 ml-6 rounded-sm px-2 py-0.5 text-sm font-medium"
		>
			{profiles.active?.modCount}
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
						{#each profiles.list as profile, index}
							<ProfilesDropdownItem {profile} {index} />
						{/each}

						<DropdownMenu.Item
							class="bg-accent-700 hover:bg-accent-600 flex cursor-pointer items-center justify-center rounded-sm py-1 text-white"
							onclick={() => (createDialogOpen = true)}
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

<CreateProfileDialog bind:open={createDialogOpen} />
