<script lang="ts">
	import { Dialog } from 'bits-ui';
	import { fade, scale } from 'svelte/transition';
	import { quartIn, quartOut } from 'svelte/easing';

	export let title: string | undefined = undefined;
	export let open: boolean;
	export let canClose: boolean = true;
	export let onClose: () => void = () => {};

	$: {
		if (!open) onClose();
	}
</script>

<Dialog.Root
	bind:open
	closeOnEscape={canClose}
	onOutsideClick={(evt) => {
		if (!canClose) evt.preventDefault();
	}}
>
	<Dialog.Portal>
		<Dialog.Overlay
			class="fixed inset-0 z-0 bg-black/60"
			transition={fade}
			transitionConfig={{ duration: 150 }}
		/>
		<Dialog.Content
			class="fixed inset-0 flex items-center justify-center pointer-events-none"
			inTransition={scale}
			inTransitionConfig={{ duration: 200, easing: quartOut, start: 0.8 }}
			outTransition={scale}
			outTransitionConfig={{ duration: 100, easing: quartIn, start: 0.8 }}
		>
			<div class="z-50 bg-gray-800 rounded-xl p-6 shadow-xl border border-gray-600 overflow-y-auto overflow-x-hidden w-full max-w-[40rem] max-h-[90%] pointer-events-auto">
				{#if title}
					<Dialog.Title class="w-full text-slate-100 font-bold text-2xl">{title}</Dialog.Title>
				{/if}

				<slot />
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
