<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { Menubar } from 'bits-ui';

	import InputField from '$lib/components/ui/InputField.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';

	import ImportR2Dialog from '$lib/components/dialogs/ImportR2Dialog.svelte';
	import ExportCodeDialog from '$lib/components/dialogs/ExportCodeDialog.svelte';
	import ImportProfileDialog from '$lib/components/dialogs/ImportProfileDialog.svelte';
	import AboutDialog from '$lib/components/dialogs/AboutDialog.svelte';
	import CreateProfileDialog from '$lib/components/dialogs/CreateProfileDialog.svelte';

	import MenubarItem from './MenubarItem.svelte';
	import MenubarMenu from './MenubarMenu.svelte';
	import MenubarSeparator from './MenubarSeparator.svelte';

	import { capitalize, fileToBase64, shortenFileSize } from '$lib/util';
	import * as api from '$lib/api';
	import { useNativeMenu } from '$lib/theme';

	import { confirm, open } from '@tauri-apps/plugin-dialog';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { open as shellOpen } from '@tauri-apps/plugin-shell';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { pushInfoToast, pushToast } from '$lib/toast';
	import { Menu, MenuItem, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';

	let importR2Open = $state(false);
	let newProfileOpen = $state(false);

	let exportCodeDialog: ExportCodeDialog;
	let importProfileDialog: ImportProfileDialog;

	let profileOperation: 'rename' | 'duplicate' = $state('rename');
	let profileOperationName = $state('');
	let profileOperationOpen = $state(false);
	let profileOperationInProgress = $state(false);

	let aboutOpen = $state(false);

	let menu: Menu | null = $state(null);

	const submenus = [
		{
			text: m.menuBar_file_title(),
			items: [
				{
					text: m.menuBar_file_item_1(),
					onclick: api.profile.openDir
				},
				{
					text: m.menuBar_file_item_2(),
					onclick: api.profile.launch.openGameDir
				},
				'',
				{
					text: m.menuBar_file_item_3(),
					onclick: api.profile.openGameLog
				},
				{
					text: m.menuBar_file_item_4(),
					onclick: api.logger.openGaleLog
				},
				'',
				{
					text: m.menuBar_file_item_5(),
					onclick: () => clearModCache(false)
				},
				{
					text: m.menuBar_file_item_6(),
					onclick: () => clearModCache(true)
				},
				{
					text: m.menuBar_file_item_7(),
					onclick: api.thunderstore.triggerModFetch
				}
			]
		},
		{
			text: m.menuBar_profile_title(),
			items: [
				{
					text: m.menuBar_profile_item_1(),
					accelerator: 'Ctrl+N',
					onclick: () => (newProfileOpen = true)
				},
				{
					text: m.menuBar_profile_item_2(),
					accelerator: 'F2',
					onclick: () => openProfileOperation('rename')
				},
				{
					text: m.menuBar_profile_item_3(),
					accelerator: 'Ctrl+D',
					onclick: () => openProfileOperation('duplicate')
				},
				'',
				{
					text: m.menuBar_profile_item_4(),
					onclick: copyModList
				},
				{
					text: m.menuBar_profile_item_5(),
					onclick: copyDebugInfo
				},
				{
					text: m.menuBar_profile_item_6(),
					onclick: copyLaunchArgs
				},
				'',
				{
					text: m.menuBar_profile_item_7(),
					onclick: () => setAllModsState(true)
				},
				{
					text: m.menuBar_profile_item_8(),
					onclick: () => setAllModsState(false)
				},
				{
					text: m.menuBar_profile_item_9(),
					onclick: uninstallDisabledMods
				},
				'',
				{
					text: m.menuBar_profile_item_10(),
					onclick: createDesktopShotcut
				}
			]
		},
		{
			text: m.menuBar_import_title(),
			items: [
				{
					text: m.menuBar_import_item_1(),
					onclick: () => importProfileDialog.openForCode()
				},
				{
					text: m.menuBar_import_item_2(),
					onclick: browseImportFile
				},
				{
					text: m.menuBar_import_item_3(),
					onclick: importLocalMod
				},
				{
					text: m.menuBar_import_item_4(),
					onclick: () => (importR2Open = true)
				}
			]
		},
		{
			text: m.menuBar_export_title(),
			items: [
				{
					text: m.menuBar_export_item_1(),
					onclick: () => exportCodeDialog.open()
				},
				{
					text: m.menuBar_export_item_2(),
					onclick: exportFile
				}
			]
		},
		{
			text: m.menuBar_window_title(),
			items: [
				{
					text: m.menuBar_window_item_1(),
					accelerator: 'Ctrl++',
					onclick: () => api.prefs.zoomWindow({ delta: 0.25 })
				},
				{
					text: m.menuBar_window_item_2(),
					accelerator: 'Ctrl+-',
					onclick: () => api.prefs.zoomWindow({ delta: -0.25 })
				},
				{
					text: m.menuBar_window_item_3(),
					accelerator: 'Ctrl+0',
					onclick: () => api.prefs.zoomWindow({ factor: 1 })
				}
			]
		},
		{
			text: m.menuBar_help_title(),
			items: [
				{
					text: m.menuBar_help_item_1(),
					onclick: () => shellOpen('https://github.com/Kesomannen/ModManager/issues/')
				},
				{
					text: m.menuBar_help_item_2(),
					onclick: () => shellOpen('https://discord.gg/sfuWXRfeTt')
				},
				{
					text: m.menuBar_help_item_3(),
					onclick: () => (aboutOpen = true)
				}
			]
		}
	];

	const appWindow = getCurrentWindow();

	async function importLocalMod() {
		let path = await open({
			title: m.menuBar_importLocalMod_title(),
			filters: [{ name: m.menuBar_importLocalMod_filters(), extensions: ['dll', 'zip'] }]
		});

		if (path === null) return;
		await api.profile.import.localMod(path);
		pushInfoToast({
			message: m.menuBar_importLocalMod_message()
		});
	}

	async function browseImportFile() {
		let path = await open({
			title: m.menuBar_browseImportFile_title(),
			filters: [{ name: m.menuBar_browseImportFile_filters(), extensions: ['r2z'] }]
		});

		if (path === null) return;
		let data = await api.profile.import.readFile(path);
		importProfileDialog.openFor({ type: 'legacy', ...data });
	}

	async function exportFile() {
		let dir = await open({
			directory: true,
			title: m.menuBar_exportFile_title()
		});

		if (dir === null) return;
		api.profile.export.file(dir);
	}

	async function setAllModsState(enable: boolean) {
		let count = await api.profile.setAllModsState(enable);
		let messageText = enable
			? m.menuBar_setAllModsState_message_enable
			: m.menuBar_setAllModsState_message_disable;
		pushInfoToast({
			message: messageText({ count: count })
		});
	}

	function openProfileOperation(operation: 'rename' | 'duplicate') {
		profileOperation = operation;
		profileOperationName = profiles.active?.name ?? m.unknown();
		profileOperationOpen = true;
	}

	async function doProfileOperation() {
		if (profileOperationInProgress) return;

		profileOperationInProgress = true;

		try {
			if (profileOperation == 'rename') {
				await api.profile.rename(profileOperationName);
				pushInfoToast({
					message: m.menuBar_doProfileOperation_rename_message({ name: profileOperationName })
				});
			} else if (profileOperation == 'duplicate') {
				await api.profile.duplicate(profileOperationName);
				pushInfoToast({
					message: m.menuBar_doProfileOperation_duplicate_message({ name: profileOperationName })
				});
			}
		} catch (e) {
			profileOperationInProgress = false;
			throw e;
		}

		profileOperationInProgress = false;
		profileOperationOpen = false;
	}

	async function createDesktopShotcut() {
		await api.profile.createDesktopShortcut();

		pushInfoToast({
			message: m.menuBar_createDesktopShotcut_message({
				name: profiles.active?.name ?? m.unknown()
			})
		});
	}

	async function uninstallDisabledMods() {
		let confirmed = await confirm(m.menuBar_uninstallDisabledMods_confirm());
		if (!confirmed) return;

		let count = await api.profile.removeDisabledMods();

		pushInfoToast({
			message: m.menuBar_uninstallDisabledMods_message({ count: count })
		});
	}

	async function copyLaunchArgs() {
		let str = await api.profile.launch.getArgs();
		await writeText(str);

		pushInfoToast({
			message: m.menuBar_copyLaunchArgs_message()
		});
	}

	async function clearModCache(soft: boolean) {
		if (!soft) {
			let result = await confirm(m.menuBar_clearModCache_confirm());

			if (!result) return;
		}

		let size = await api.profile.install.clearDownloadCache(soft);
		let messageText = soft
			? m.menuBar_clearModCache_message_unsed
			: m.menuBar_clearModCache_message;
		pushInfoToast({
			message: messageText({ size: shortenFileSize(size) })
		});
	}

	async function copyModList() {
		await api.profile.export.copyDependencyStrings();
		pushInfoToast({
			message: m.menuBar_copyModList_message()
		});
	}

	async function copyDebugInfo() {
		await api.profile.export.copyDebugInfo();
		pushInfoToast({
			message: m.menuBar_copyDebugInfo_message()
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
			let data = await api.profile.import.readBase64(base64);
			importProfileDialog.openFor({ type: 'legacy', ...data });
		} else if (file.name.endsWith('.zip')) {
			if (profiles.activeLocked) {
				pushToast({
					type: 'error',
					name: m.menuBar_handleFileDrop_activeLocked_title(),
					message: m.menuBar_handleFileDrop_activeLocked_message()
				});
				return;
			}

			await api.profile.import.localModBase64(base64);
			pushInfoToast({
				message: m.menuBar_handleFileDrop_message()
			});
		}
	}

	$effect(() => {
		if (menu) {
			appWindow.setDecorations(useNativeMenu.current);

			if (useNativeMenu.current) {
				menu.setAsAppMenu();
			} else {
				Menu.new().then((menu) => menu.setAsAppMenu());
			}
		}
	});

	const hotkeys: { [key: string]: () => void } = {
		'+': () => api.prefs.zoomWindow({ delta: 0.25 }),
		'-': () => api.prefs.zoomWindow({ delta: -0.25 }),
		'0': () => api.prefs.zoomWindow({ factor: 1 }),
		n: () => (newProfileOpen = true),
		d: () => openProfileOperation('duplicate')
	};

	onMount(async () => {
		document.onkeydown = ({ key, ctrlKey }) => {
			if (useNativeMenu.current) return;

			if (key === 'F2') {
				openProfileOperation('rename');
				return;
			}

			if (!ctrlKey) return;

			let hotkey = hotkeys[key];
			if (hotkey !== undefined) hotkey();
		};

		let separator = await PredefinedMenuItem.new({
			item: 'Separator'
		});

		let nativeMenus = await Promise.all(
			submenus.map(
				async (menu) =>
					await Submenu.new({
						text: menu.text,
						items: await Promise.all(
							menu.items.map(async (item) =>
								typeof item === 'string'
									? separator
									: await MenuItem.new({
											action: item.onclick,
											...item
										})
							)
						)
					})
			)
		);

		menu = await Menu.new({
			items: nativeMenus
		});
	});
</script>

<svelte:body
	ondragenter={(evt) => evt.preventDefault()}
	ondragover={(evt) => evt.preventDefault()}
	ondrop={handleFileDrop}
/>

<header
	data-tauri-drag-region
	class="bg-primary-800 flex h-8 shrink-0"
	class:hidden={useNativeMenu.current}
>
	<Menubar.Root class="flex items-center py-1">
		<img src="favicon.png" alt="Gale logo" class="mr-2 ml-4 h-5 w-5 opacity-50" />
		{#each submenus as submenu}
			<MenubarMenu label={submenu.text}>
				{#each submenu.items as item}
					{#if typeof item === 'string'}
						<MenubarSeparator />
					{:else}
						<MenubarItem onclick={item.onclick} text={item.text} />
					{/if}
				{/each}
			</MenubarMenu>
		{/each}
	</Menubar.Root>

	{#snippet button(className: string, icon: string, onclick: () => void)}
		<button class={[className, 'group hover:bg-primary-700 px-3 py-1.5']} {onclick}>
			<Icon {icon} class="text-primary-500 group-hover:text-white" />
		</button>
	{/snippet}

	{@render button('hover:bg-primary-700 ml-auto', 'mdi:minimize', appWindow.minimize)}
	{@render button('hover:bg-primary-700', 'mdi:maximize', appWindow.toggleMaximize)}
	{@render button('hover:bg-red-700', 'mdi:close', appWindow.close)}
</header>

<Dialog
	title={m[`menuBar_dialog_title_${profileOperation}`]()}
	canClose={!profileOperationInProgress}
	bind:open={profileOperationOpen}
>
	<p class="text-primary-300 mb-1">
		{m[`menuBar_dialog_content_${profileOperation}`]()}
	</p>

	<InputField
		bind:value={profileOperationName}
		placeholder={m.menuBar_dialog_input_placeholder()}
		size="lg"
		class="w-full"
		onsubmit={doProfileOperation}
	/>

	<div class="mt-2 ml-auto flex justify-end gap-2">
		{#if !profileOperationInProgress}
			<Button color="primary" onclick={() => (profileOperationOpen = false)}
				>{m.menuBar_dialog_button_cancel()}</Button
			>
		{/if}

		<Button
			color="accent"
			loading={profileOperationInProgress}
			onclick={doProfileOperation}
			icon={profileOperation === 'duplicate' ? 'mdi:content-duplicate' : 'mdi:edit'}
		>
			{m[`menuBar_dialog_button_accent_${profileOperation}`]()}
		</Button>
	</div>
</Dialog>

<AboutDialog bind:open={aboutOpen} />
<ImportR2Dialog bind:open={importR2Open} />
<CreateProfileDialog bind:open={newProfileOpen} />
<ExportCodeDialog bind:this={exportCodeDialog} />
<ImportProfileDialog bind:this={importProfileDialog} />
