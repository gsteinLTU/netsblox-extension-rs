(function () {    
    class ExampleExtension extends Extension {
        constructor(ide) {
            super('ExampleExtension');
        }

        onOpenRole() {

        }

        getSettings() {
            return [

            ];
        }

        getMenu() {
            return {

            };
        }

        getCategories() {
            return [

            ];
        }

        getPalette() {
            return [
				new Extension.PaletteCategory(
					'control',
					[
						new Extension.Palette.Block('logHelloWorld'),
					],
					SpriteMorph
				),
				new Extension.PaletteCategory(
					'control',
					[
						new Extension.Palette.Block('logHelloWorld'),
					],
					StageMorph
				),

            ];
        }

        getBlocks() {
            return [
				new Extension.Block(
					'logHelloWorld',
					'command',
					'control',
					'Log Hello World!',
					[],
					function () { hello_world() }
				).for(SpriteMorph, StageMorph),

            ];
        }

        getLabelParts() {
            return [

            ];
        }

    }

    NetsBloxExtensions.register(ExampleExtension);
})();