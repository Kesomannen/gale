<script lang="ts">
	import Icon from '@iconify/svelte';
	import { DropdownMenu } from 'bits-ui';
	import ProfilesDropdownItem from './ProfilesDropdownItem.svelte';
	import CreateProfileDialog from '$lib/components/dialogs/CreateProfileDialog.svelte';
	import { fade, fly } from 'svelte/transition';
	import { dropIn, dropOut } from '$lib/transitions';
	import DropdownArrow from '../ui/DropdownArrow.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../ui/Button.svelte';

	let open = $state(false);
	let createDialogOpen = $state(false);
</script>

<DropdownMenu.Root bind:open>
	<DropdownMenu.Trigger
		class="group text-primary-300 group-hover:text-primary-200 hover:bg-primary-800 flex shrink items-center overflow-hidden rounded-lg px-4 py-2"
	>
		<span class="mr-auto shrink truncate font-semibold">
			{profiles.active?.name}
		</span>

		<div
			class="bg-primary-800 group-hover:bg-primary-700 mr-2 ml-6 rounded-sm px-1.5 py-1 text-xs font-medium"
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
						class="border-primary-600 bg-primary-800 z-30 flex max-h-[80lvh] min-w-40 flex-col gap-0.5 overflow-y-auto rounded-lg border p-1 shadow-lg"
						in:fly={dropIn}
						out:fade={dropOut}
					>
						{#each profiles.list as profile, index (profile.id)}
							<ProfilesDropdownItem {profile} {index} />
						{/each}

						<DropdownMenu.Item class="mt-1 flex">
							<Button onclick={() => (createDialogOpen = true)} icon="mdi:plus" class="grow">
								{m.profilesDropdown_button()}
							</Button>
						</DropdownMenu.Item>
					</div>
				{/if}
			</div>
		{/snippet}
	</DropdownMenu.Content>
</DropdownMenu.Root>

<CreateProfileDialog bind:open={createDialogOpen} />
