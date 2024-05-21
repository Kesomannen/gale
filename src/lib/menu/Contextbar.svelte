<script lang="ts">
	import { confirm } from '@tauri-apps/api/dialog';

	import Popup from '$lib/components/Popup.svelte';
	import {
		activeProfileIndex,
		currentGame,
		currentProfile,
		profileNames,
		refreshProfiles,
		setActiveProfile
	} from '$lib/profile';
	import { invokeCommand } from '$lib/invoke';

	import Icon from '@iconify/svelte';
	import { Button, Dialog, DropdownMenu } from 'bits-ui';
	import SelectGamePopup from './SelectGamePopup.svelte';

	let launchGamePopupOpen = false;

	let gamesOpen = false;
	let profilesOpen = false;

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
		class="flex items-center flex-shrink-0 pl-6 pr-8 border-r border-gray-600 text-green-400 hover:text-green-400 hover:bg-gray-800 cursor-default"
		on:click={() => {
			invokeCommand('launch_game').then(() => (launchGamePopupOpen = true));
		}}
	>
		<Icon icon="mdi:play-circle" class="text-xl mr-2" />
		<div class="font-medium">Launch game</div>
	</Button.Root>

	<Button.Root
		on:click={() => (gamesOpen = !gamesOpen)}
		class="flex items-center justify-between pl-2 pr-4 group border-r border-gray-600 hover:bg-gray-800 text-slate-300 group-hover:text-slate-200 cursor-default"
	>
		{#if $currentGame}
			<img
				src="games/{$currentGame.id}.png"
				class="max-w-8 max-h-8 rounded mr-2"
				alt={$currentGame.displayName}
			/>

			{$currentGame.displayName}
		{:else}
			Loading...
		{/if}

		<Icon
			icon="mdi:menu"
			class="text-slate-300 group-hover:text-slate-200 text-xl transition-all flex-shrink-0 ml-6"
		/>
	</Button.Root>

	<DropdownMenu.Root bind:open={profilesOpen}>
		<DropdownMenu.Trigger
			class="flex flex-shrink items-center justify-between min-w-40 pl-6 pr-4 group border-r border-gray-600 
						text-slate-300 group-hover:text-slate-200 hover:bg-gray-800 cursor-default"
		>
			<div class="flex-shrink truncate">
				{$currentProfile}
			</div>

			<Icon
				icon="mdi:expand-more"
				class="flex-shrink-0 text-xl transition-all transform origin-center ml-6 {profilesOpen
					? 'rotate-180'
					: 'rotate-0'}"
			/>
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 rounded-lg border border-gray-600"
		>
			{#each profileNames as profile, i}
				<DropdownMenu.Item
					class="flex items-center px-3 py-1 truncate text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
					on:click={() => {
						setActiveProfile(i);
						profilesOpen = false;
					}}
				>
					<span class="flex-shrink truncate mr-5">
						{profile}
					</span>

					<div class="ml-auto inline-flex"> 
						{#if i == activeProfileIndex}
							<Icon icon="mdi:check" class=" text-green-400 text-lg" />
						{/if}
						<Button.Root
							class="text-red-500 hover:text-red-400 text-lg ml-1"
							on:click={(evt) => {
								evt.stopPropagation();
								deleteProfile(i);
								profilesOpen = false;
							}}
						>
							<Icon icon="mdi:delete" />
						</Button.Root>
					</div>
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Content>
	</DropdownMenu.Root>
</div>

<Popup title="Launching {$currentGame?.displayName}..." bind:open={launchGamePopupOpen}>
	<Dialog.Description class="text-slate-400">
		If the game is taking a while to start, it's probably because Steam is starting
		up.
	</Dialog.Description>
</Popup>

<SelectGamePopup bind:open={gamesOpen} />
