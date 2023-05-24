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
						new Extension.Palette.Block('logHelloName'),
						new Extension.Palette.Block('logHelloWorld'),
					],
					SpriteMorph
				),
				new Extension.PaletteCategory(
					'control',
					[
						new Extension.Palette.Block('logHelloName'),
						new Extension.Palette.Block('logHelloWorld'),
					],
					StageMorph
				),

            ];
        }

        getBlocks() {
            return [
				new Extension.Block(
					'logHelloName',
					'command',
					'control',
					'Log Hello %name',
					[],
					function (name) { hello_name(name) }
				).for(SpriteMorph, StageMorph),
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
				new Extension.LabelPart(
					'%name',
					() => {
						const part = new InputSlotMorph(
							null, // text
							false, // non-numeric
							null,
							false
						);
						return part;
					}
				),

            ];
        }

    }

    NetsBloxExtensions.register(ExampleExtension);
})();