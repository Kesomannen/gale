<script lang="ts">
	import { Button, Dialog } from 'bits-ui';
	import { fade, fly, scale } from 'svelte/transition';
	import { quadOut, quartIn, quartOut } from 'svelte/easing';
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
				class="dialog pointer-events-auto relative z-50 max-h-[85%] overflow-y-auto overflow-x-hidden rounded-xl border border-slate-600 bg-slate-800 p-6 shadow-xl"
				class:large
			>
				{#if canClose}
					<Button.Root
						class="absolute right-5 top-5 rounded-md p-0.5 text-3xl text-slate-400 hover:bg-red-600/80 hover:text-red-100"
						on:click={close}
					>
						<Icon icon="mdi:close" />
					</Button.Root>
				{/if}

				{#if title}
					<Dialog.Title class="w-full break-words pr-10 text-2xl font-bold text-white"
						>{title}</Dialog.Title
					>
				{/if}

				<slot />
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>

<style lang="postcss">
	.dialog {
		width: 85%;
		max-width: 35rem;
	}

	.dialog.large {
		max-width: 60rem;
	}
</style>
