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

	import { capitalize, fileToBase64, shortenFileSize } from '$lib/util';
	import { invokeCommand } from '$lib/invoke';
	import type { ImportData } from '$lib/models';
	import { activeProfile, refreshProfiles } from '$lib/stores';

	import { confirm, open } from '@tauri-apps/plugin-dialog';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { open as shellOpen } from '@tauri-apps/plugin-shell';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { pushInfoToast, pushToast } from '$lib/toast';

	let importR2Open = false;
	let newProfileOpen = false;

	let exportCodePopup: ExportCodePopup;
	let importProfilePopup: ImportProfilePopup;

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

		pushInfoToast({
			message: 'Imported local mod into profile.'
		});
	}

	async function browseImportFile() {
		let path = await open({
			title: 'Select the file to import',
			filters: [{ name: 'Profile file', extensions: ['r2z'] }]
		});

		if (path === null) return;
		let data = await invokeCommand<ImportData>('read_profile_file', { path });
		importProfilePopup.openFor(data);
	}

	async function exportFile() {
		let dir = await open({
			directory: true,
			title: 'Select the folder to export the profile to'
		});

		if (dir === null) return;
		invokeCommand('export_file', { dir });
	}

	async function setAllModsState(enable: boolean) {
		let count = await invokeCommand<number>('set_all_mods_state', { enable });

		pushInfoToast({
			message: `${enable ? 'Enabled' : 'Disabled'} ${count} mods.`
		});

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
				pushInfoToast({
					message: `Renamed profile to ${profileOperationName}.`
				});
			} else if (profileOperation == 'duplicate') {
				await invokeCommand('duplicate_profile', { name: profileOperationName });
				pushInfoToast({
					message: `Duplicated profile to ${profileOperationName}.`
				});
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

		let count = await invokeCommand<number>('remove_disabled_mods');

		pushInfoToast({
			message: `Uninstalled ${count} disabled mods.`
		});

		await refreshProfiles();
	}

	async function zoom(value: { delta: number } | { factor: number }) {
		await invokeCommand('zoom_window', { value });
	}

	async function copyLaunchArgs() {
		let str = await invokeCommand<string>('get_launch_args');
		await writeText(str);

		pushInfoToast({
			message: `Copied launch arguments to clipboard.`
		});
	}

	async function clearModCache(soft: boolean) {
		if (!soft) {
			let result = await confirm(
				"Are you sure you want to delete all cached mods? This could potentially double the disk space used by installed mods. Only proceed if you know what you're doing!"
			);

			if (!result) return;
		}

		let size = await invokeCommand<number>('clear_download_cache', { soft });
		pushInfoToast({
			message: `Deleted${soft ? ' unused' : ''} mod cache (cleared ${shortenFileSize(size)}).`
		});
	}

	async function copyModList() {
		await invokeCommand('copy_dependency_strings');
		pushInfoToast({
			message: 'Copied mod list to clipboard.'
		});
	}

	async function copyDebugInfo() {
		await invokeCommand('copy_debug_info');
		pushInfoToast({
			message: 'Copied debug info to clipboard.'
		});
	}

	async function handleFileDrop(evt: DragEvent) {
		evt.preventDefault();
		if (evt.dataTransfer === null) return;

		let file: File | null;
		if (evt.dataTransfer.items) {
			let files = [...evt.dataTransfer.items].filter((item) => item.kind == 'file');
			if (files.length === 0) return;
			file = files[0].getAsFile();
		} else {
			file = [...evt.dataTransfer.items][0];
		}

		if (file === null) return;
		let base64 = await fileToBase64(file);

		if (file.name.endsWith('.r2z')) {
			let data = await invokeCommand<ImportData>('read_profile_base64', { base64 });
			importProfilePopup.openFor(data);
		} else if (file.name.endsWith('.zip')) {
			await invokeCommand('import_local_mod_base64', { base64 });
			await refreshProfiles();

			pushInfoToast({
				message: 'Imported local mod into profile.'
			});
		}
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

<svelte:body
	on:dragenter={(evt) => evt.preventDefault()}
	on:dragover={(evt) => evt.preventDefault()}
	on:drop={handleFileDrop}
/>

<header data-tauri-drag-region class="bg-primary-800 flex h-8 shrink-0">
	<Menubar.Root class="flex items-center py-1">
		<img src="favicon.png" alt="Gale logo" class="mr-2 ml-4 h-5 w-5 opacity-50" />
		<MenubarMenu label="File">
			<MenubarItem on:click={() => invokeCommand('open_profile_dir')} text="Open profile folder" />
			<MenubarItem on:click={() => invokeCommand('open_game_dir')} text="Open game folder" />
			<MenubarSeparator />
			<MenubarItem on:click={() => invokeCommand('open_game_log')} text="Open game log" />
			<MenubarItem on:click={() => invokeCommand('open_gale_log')} text="Open Gale log" />
			<MenubarSeparator />
			<MenubarItem on:click={() => clearModCache(false)} text="Clear mod cache" />
			<MenubarItem on:click={() => clearModCache(true)} text="Clear unused mod cache" />
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
			<MenubarItem on:click={copyModList} text="Copy mod list" />
			<MenubarItem on:click={copyDebugInfo} text="Copy debug info" />
			<MenubarItem on:click={copyLaunchArgs} text="Copy launch arguments" />
			<MenubarSeparator />
			<MenubarItem on:click={() => setAllModsState(true)} text="Enable all mods" />
			<MenubarItem on:click={() => setAllModsState(false)} text="Disable all mods" />
			<MenubarItem on:click={uninstallDisabledMods} text="Uninstall disabled mods" />
		</MenubarMenu>
		<MenubarMenu label="Import">
			<MenubarItem on:click={() => importProfilePopup.openForCode()} text="...profile from code" />
			<MenubarItem on:click={browseImportFile} text="...profile from file" />
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

	<Button.Root class="group hover:bg-primary-700 ml-auto px-3 py-1.5" on:click={appWindow.minimize}>
		<Icon icon="mdi:minimize" class="text-primary-500 group-hover:text-white" />
	</Button.Root>
	<Button.Root class="group hover:bg-primary-700 px-3 py-1.5" on:click={appWindow.toggleMaximize}>
		<Icon icon="mdi:maximize" class="text-primary-500 group-hover:text-white" />
	</Button.Root>
	<Button.Root class="group px-3 py-1.5 hover:bg-red-700" on:click={appWindow.close}>
		<Icon icon="mdi:close" class="text-primary-500 group-hover:text-white" />
	</Button.Root>
</header>

<Popup
	title="{capitalize(profileOperation)} profile"
	canClose={!profileOperationInProgress}
	bind:open={profileOperationOpen}
>
	<p class="text-primary-300 mb-1">
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
	<div class="mt-2 ml-auto flex justify-end gap-2">
		{#if !profileOperationInProgress}
			<BigButton color="primary" on:click={() => (profileOperationOpen = false)}>Cancel</BigButton>
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
<ImportProfilePopup bind:this={importProfilePopup} />
