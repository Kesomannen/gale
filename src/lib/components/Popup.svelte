<script lang="ts">
	import { Button, Dialog } from 'bits-ui';
	import { fade } from 'svelte/transition';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import { popupTransition } from '$lib/transitions';

	export let open: boolean;
	export let title: string | null = null;
	export let confirmClose: { message: string } | null = null;
	export let canClose: boolean = true;
	export let large: boolean = false;
	export let onClose: () => void = () => {};

	async function close(evt: UIEvent) {
		if (!canClose) {
			evt.preventDefault();
			return;
		}

		if (confirmClose) {
			evt.preventDefault();
			let result = await confirm(confirmClose.message);
			if (!result) {
				return;
			}
		}

		open = false;
		onClose();
	}
</script>

<Dialog.Root
	bind:open
	closeOnEscape={canClose && confirmClose === null}
	closeOnOutsideClick={canClose && confirmClose === null}
	onOpenChange={(open) => {
		if (!open) {
			onClose();
		}
	}}
>
	<Dialog.Portal>
		<Dialog.Overlay
			data-tauri-drag-region
			class="fixed inset-0 z-0 rounded-lg bg-black/60"
			transition={fade}
			transitionConfig={{ duration: 100 }}
		/>
		<Dialog.Content
			class="pointer-events-none fixed inset-0 flex items-center justify-center"
			{...popupTransition}
		>
			<div
				class="w-[85%] {large
					? 'max-w-[60rem]'
					: 'max-w-[35rem]'} border-primary-600 bg-primary-800 pointer-events-auto relative z-50 max-h-[85%] overflow-x-hidden overflow-y-auto rounded-xl border p-6 shadow-xl"
			>
				{#if canClose}
					<Button.Root
						class="text-primary-400 hover:bg-primary-700 hover:text-primary-300 absolute top-5 right-5 rounded-md p-0.5 text-3xl"
						on:click={close}
					>
						<Icon icon="mdi:close" />
					</Button.Root>
				{/if}

				{#if title}
					<Dialog.Title class="w-full pr-10 text-2xl font-bold break-words text-white"
						>{title}</Dialog.Title
					>
				{/if}

				<slot />
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
