<script lang="ts">
	import { confirm } from '@tauri-apps/api/dialog';

	import Popup from '$lib/Popup.svelte';
	import {
		activeProfileIndex,
		currentProfile,
		profileNames,
		refreshProfiles,
		setActiveProfile
	} from '$lib/profile';
	import { invokeCommand } from '$lib/invoke';
	
	import Icon from '@iconify/svelte';
	import { Button, Dialog, DropdownMenu } from 'bits-ui';

	import { slide } from 'svelte/transition';

	let profilesOpen = false;
	let startingGame = false;

	function deleteProfile(index: number) {
		confirm(`Are you sure you want to delete ${profileNames[index]}?`, {
			title: 'Delete profile'
		}).then(async (result) => {
			if (result) {
				await invokeCommand('delete_profile', { index });
				refreshProfiles();
			}
		});
	}
</script>

<div class="h-12 flex flex-row flex-shrink-0 bg-gray-900 border-b border-t border-gray-600">
	<Button.Root
		class="flex flex-row items-center px-6 w-48 border-r border-gray-600 relative hover:bg-gray-800 cursor-default group"
		on:click={() => {
			invokeCommand('start_game').then(() => (startingGame = true));
		}}
	>
		<Icon icon="mdi:play-circle" class="text-green-400 group-hover:text-green-300 text-xl mr-2" />
		<div class="text-green-400 group-hover:text-green-300 font-medium">Run game</div>
	</Button.Root>

	<DropdownMenu.Root bind:open={profilesOpen}>
		<DropdownMenu.Trigger
			class="flex flex-row items-center px-6 w-48 group border-r border-gray-600 relative hover:bg-gray-800 cursor-default"
		>
			<div class="text-slate-300 group-hover:text-slate-200 truncate mr-4">
				{$currentProfile}
			</div>
			<Icon
				icon="mdi:expand-more"
				class="
                text-slate-300 group-hover:text-slate-200 text-xl absolute right-4 transition-all
                transform origin-center {profilesOpen ? 'rotate-180' : 'rotate-0'}"
			/>
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 w-48 rounded-lg border border-gray-600"
			transition={slide}
			transitionConfig={{ duration: 100 }}
		>
			{#each profileNames as profile, i}
				<DropdownMenu.Item
					class="flex items-center px-3 py-1 truncate text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
					disabled={i == activeProfileIndex}
					on:click={() => {
						setActiveProfile(i);
						profilesOpen = false;
					}}
				>
					{profile}

					<div class="ml-auto inline-flex">
						{#if i == activeProfileIndex}
							<Icon icon="mdi:check" class=" text-green-400 text-lg" />
						{/if}
						<Button.Root class="group" on:click={() => deleteProfile(i)}>
							<Icon icon="mdi:delete" class="text-red-400 hover:text-red-300 text-lg ml-2" />
						</Button.Root>
					</div>
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Content>
	</DropdownMenu.Root>
</div>

<Popup title="Starting game..." bind:open={startingGame}>
	<Dialog.Description class="text-slate-400">
		Click outside this window to continue modding. If it's taking a while, it's probably because
		Steam is starting up.
	</Dialog.Description>
</Popup>
