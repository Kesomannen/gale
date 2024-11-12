<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { Button, Menubar } from 'bits-ui';

	import MenubarItem from '$lib/menu/MenubarItem.svelte';

	import InputField from '$lib/components/InputField.svelte';
	import BigButton from '$lib/components/BigButton.svelte';
	import Popup from '$lib/components/Popup.svelte';

	import ImportR2Popup from '$lib/import/ImportR2Popup.svelte';
	import ExportCodePopup from '$lib/import/ExportCodePopup.svelte';
	import ImportProfilePopup from '$lib/import/ImportProfilePopup.svelte';

	import AboutPopup from './AboutPopup.svelte';
	import MenubarMenu from './MenubarMenu.svelte';
	import NewProfilePopup from './NewProfilePopup.svelte';
	import MenubarSeparator from './MenubarSeparator.svelte';

	import { capitalize } from '$lib/util';
	import { invokeCommand } from '$lib/invoke';
	import type { ImportData } from '$lib/models';
	import { activeProfile, refreshProfiles } from '$lib/stores';

	import { confirm, open } from '@tauri-apps/plugin-dialog';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { open as shellOpen } from '@tauri-apps/plugin-shell';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';

	let importR2Open = false;
	let newProfileOpen = false;
	let exportCodePopup: ExportCodePopup;

	let importProfileOpen = false;
	let importProfileData: ImportData | null = null;

	let profileOperation: 'rename' | 'duplicate' = 'rename';
	let profileOperationName = '';
	let profileOperationOpen = false;
	let profileOperationInProgress = false;

	let aboutOpen = false;

	const appWindow = getCurrentWindow();

	async function importLocalMod() {
		let path = await open({
			title: 'Select the mod file to import',
			filters: [{ name: 'Dll or zip', extensions: ['dll', 'zip'] }]
		});

		if (path === null) return;
		await invokeCommand('import_local_mod', { path });
		await refreshProfiles();
	}

	async function importFile() {
		let path = await open({
			title: 'Select the file to import',
			filters: [{ name: 'Profile file', extensions: ['r2z'] }]
		});

		if (path === null) return;
		let data = await invokeCommand<ImportData>('import_file', { path });

		importProfileData = data;
		importProfileOpen = true;
	}

	async function exportFile() {
		let dir = await open({
			directory: true,
			title: 'Select the directory to export the profile to'
		});

		if (dir === null) return;
		invokeCommand('export_file', { dir });
	}

	async function setAllModsState(enable: boolean) {
		await invokeCommand('set_all_mods_state', { enable });
		activeProfile.update((profile) => profile);
	}

	function openProfileOperation(operation: 'rename' | 'duplicate') {
		profileOperation = operation;
		profileOperationName = $activeProfile?.name ?? 'Unknown';
		profileOperationOpen = true;
	}

	async function doProfileOperation() {
		if (profileOperationInProgress) return;

		profileOperationInProgress = true;

		try {
			if (profileOperation == 'rename') {
				await invokeCommand('rename_profile', { name: profileOperationName });
			} else if (profileOperation == 'duplicate') {
				await invokeCommand('duplicate_profile', { name: profileOperationName });
			}
		} catch (e) {
			profileOperationInProgress = false;
			throw e;
		}

		await refreshProfiles();
		profileOperationInProgress = false;
		profileOperationOpen = false;
	}

	async function uninstallDisabledMods() {
		let confirmed = await confirm('Are you sure you want to uninstall all disabled mods?');
		if (!confirmed) return;

		await invokeCommand<number>('remove_disabled_mods');
		await refreshProfiles();
	}

	async function zoom(value: { delta: number } | { factor: number }) {
		await invokeCommand('zoom_window', { value });
	}

	async function copyLaunchArgs() {
		let str = await invokeCommand<string>('get_launch_args');
		writeText(str);
	}

	async function clearModCache() {
		let result = await confirm(
			"Are you sure you want to delete all cached mods? This could potentially double the disk space used by installed mods. Only proceed if you know what you're doing!"
		);

		if (!result) return;

		await invokeCommand('clear_download_cache', { soft: false });
	}

	const hotkeys: { [key: string]: () => void } = {
		'+': () => zoom({ delta: 0.25 }),
		'-': () => zoom({ delta: -0.25 }),
		'0': () => zoom({ factor: 1 }),
		n: () => (newProfileOpen = true),
		d: () => openProfileOperation('duplicate')
	};

	onMount(() => {
		document.onkeydown = ({ key, ctrlKey }) => {
			if (key === 'F2') {
				openProfileOperation('rename');
				return;
			}

			if (!ctrlKey) return;

			const hotkey = hotkeys[key];
			if (hotkey !== undefined) hotkey();
		};
	});
</script>

<header data-tauri-drag-region class="flex h-8 flex-shrink-0 bg-slate-800">
	<Menubar.Root class="flex items-center py-1">
		<img src="favicon.png" alt="Gale logo" class="ml-4 mr-2 h-5 w-5 opacity-50" />
		<MenubarMenu label="File">
			<MenubarItem
				on:click={() => invokeCommand('open_profile_dir')}
				text="Open profile directory"
			/>
			<MenubarItem on:click={() => invokeCommand('open_game_dir')} text="Open game directory" />
			<MenubarSeparator />
			<MenubarItem on:click={() => invokeCommand('open_game_log')} text="Open game log" />
			<MenubarItem on:click={() => invokeCommand('open_gale_log')} text="Open Gale log" />
			<MenubarSeparator />
			<MenubarItem on:click={clearModCache} text="Clear mod cache" />
			<MenubarItem
				on:click={() => invokeCommand('clear_download_cache', { soft: true })}
				text="Clear unused mod cache"
			/>
			<MenubarItem on:click={() => invokeCommand('trigger_mod_fetch')} text="Fetch mods" />
		</MenubarMenu>
		<MenubarMenu label="Profile">
			<MenubarItem
				on:click={() => (newProfileOpen = true)}
				text="Create new profile"
				key="Ctrl N"
			/>
			<MenubarItem
				on:click={() => openProfileOperation('rename')}
				text="Rename active profile"
				key="F2"
			/>
			<MenubarItem
				on:click={() => openProfileOperation('duplicate')}
				text="Duplicate active profile"
				key="Ctrl D"
			/>
			<MenubarSeparator />
			<MenubarItem on:click={() => invokeCommand('copy_dependency_strings')} text="Copy mod list" />
			<MenubarItem on:click={() => invokeCommand('copy_debug_info')} text="Copy debug info" />
			<MenubarItem on:click={copyLaunchArgs} text="Copy launch arguments" />
			<MenubarSeparator />
			<MenubarItem on:click={() => setAllModsState(true)} text="Enable all mods" />
			<MenubarItem on:click={() => setAllModsState(false)} text="Disable all mods" />
			<MenubarItem on:click={uninstallDisabledMods} text="Uninstall disabled mods" />
		</MenubarMenu>
		<MenubarMenu label="Import">
			<MenubarItem on:click={() => (importProfileOpen = true)} text="...profile from code" />
			<MenubarItem on:click={importFile} text="...profile from file" />
			<MenubarItem on:click={importLocalMod} text="...local mod" />
			<MenubarItem on:click={() => (importR2Open = true)} text="...profiles from r2modman" />
		</MenubarMenu>
		<MenubarMenu label="Export">
			<MenubarItem on:click={() => exportCodePopup.open()} text="...profile as code" />
			<MenubarItem on:click={exportFile} text="...profile as file" />
		</MenubarMenu>
		<MenubarMenu label="Window">
			<MenubarItem
				on:click={() => invokeCommand('zoom_window', { value: { delta: 0.25 } })}
				text="Zoom in"
				key="Ctrl +"
			/>
			<MenubarItem
				on:click={() => invokeCommand('zoom_window', { value: { delta: -0.25 } })}
				text="Zoom out"
				key="Ctrl -"
			/>
			<MenubarItem
				on:click={() => invokeCommand('zoom_window', { value: { factor: 1 } })}
				text="Reset zoom"
				key="Ctrl 0"
			/>
		</MenubarMenu>
		<MenubarMenu label="Help">
			<MenubarItem
				on:click={() => shellOpen('https://github.com/Kesomannen/ModManager/issues/')}
				text="Report a bug"
			/>
			<MenubarItem
				on:click={() => shellOpen('https://discord.gg/sfuWXRfeTt')}
				text="Join discord server"
			/>
			<MenubarItem on:click={() => (aboutOpen = true)} text="About Gale" />
		</MenubarMenu>
	</Menubar.Root>

	<Button.Root class="group ml-auto px-3 py-1.5 hover:bg-slate-700" on:click={appWindow.minimize}>
		<Icon icon="mdi:minimize" class="text-slate-500 group-hover:text-white" />
	</Button.Root>
	<Button.Root class="group px-3 py-1.5 hover:bg-slate-700" on:click={appWindow.toggleMaximize}>
		<Icon icon="mdi:maximize" class="text-slate-500 group-hover:text-white" />
	</Button.Root>
	<Button.Root class="group px-3 py-1.5 hover:bg-red-700" on:click={appWindow.close}>
		<Icon icon="mdi:close" class="text-slate-500 group-hover:text-white" />
	</Button.Root>
</header>

<Popup
	title="{capitalize(profileOperation)} profile"
	canClose={!profileOperationInProgress}
	bind:open={profileOperationOpen}
>
	<p class="mb-1 text-slate-300">
		{profileOperation == 'duplicate'
			? 'Enter a name for the duplicated profile:'
			: 'Enter a new name for the profile:'}
	</p>
	<InputField
		bind:value={profileOperationName}
		placeholder="Enter name..."
		size="lg"
		class="w-full"
		on:submit={doProfileOperation}
	/>
	{#if profileOperation == 'duplicate'}
		<p class="mt-2 text-sm text-slate-400">
			This process might take up to a minute depending on the size of the profile, please be
			patient.
		</p>
	{/if}
	<div class="ml-auto mt-2 flex justify-end gap-2">
		{#if !profileOperationInProgress}
			<BigButton color="slate" on:click={() => (profileOperationOpen = false)}>Cancel</BigButton>
		{/if}
		<BigButton
			color="accent"
			fontWeight="medium"
			disabled={profileOperationInProgress}
			on:click={doProfileOperation}
		>
			{#if profileOperationInProgress}
				<Icon icon="mdi:loading" class="my-1 animate-spin text-lg" />
			{:else}
				{capitalize(profileOperation)}
			{/if}
		</BigButton>
	</div>
</Popup>

<AboutPopup bind:open={aboutOpen} />
<ImportR2Popup bind:open={importR2Open} />
<NewProfilePopup bind:open={newProfileOpen} />
<ExportCodePopup bind:this={exportCodePopup} />
<ImportProfilePopup bind:open={importProfileOpen} bind:data={importProfileData} />
