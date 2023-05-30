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
				new Extension.Category('Hello World', new Color(100, 149, 237)),

            ];
        }

        getPalette() {
            return [
				new Extension.PaletteCategory(
					'Hello World',
					[
						new Extension.Palette.Block('logHelloWorld'),
						new Extension.Palette.Block('logHelloName'),
					],
					SpriteMorph
				),
				new Extension.PaletteCategory(
					'Hello World',
					[
						new Extension.Palette.Block('logHelloWorld'),
						new Extension.Palette.Block('logHelloName'),
					],
					StageMorph
				),
				new Extension.PaletteCategory(
					'operators',
					[
						new Extension.Palette.Block('isEven'),
						new Extension.Palette.Block('repeatString'),
					],
					SpriteMorph
				),
				new Extension.PaletteCategory(
					'operators',
					[
						new Extension.Palette.Block('isEven'),
						new Extension.Palette.Block('repeatString'),
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
					'Hello World',
					'Log Hello World!',
					[],
					function () { ExampleExtension_fns.hello_world() }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'isEven',
					'predicate',
					'operators',
					'is %num even?',
					[],
					function (num) { ExampleExtension_fns.is_even(num) }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'logHelloName',
					'command',
					'Hello World',
					'Log Hello %name',
					[],
					function (name) { ExampleExtension_fns.hello_name(name) }
				).for(SpriteMorph, StageMorph),
				new Extension.Block(
					'repeatString',
					'reporter',
					'operators',
					'Repeat %text for %times times',
					[],
					function (text, times) { ExampleExtension_fns.repeat_text(text, times) }
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
				new Extension.LabelPart(
					'%text',
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
				new Extension.LabelPart(
					'%times',
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
				new Extension.LabelPart(
					'%num',
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
    let path = document.currentScript.src;
    path = path.substring(0, path.lastIndexOf("/"));
    var s = document.createElement('script');
    s.type = "module";
    s.innerHTML = `import init, {hello_world, hello_name, repeat_text, is_even} from '${path}/pkg/netsblox_extension_rs.js';
    
    
        await init();

        window.ExampleExtension_fns = {};
		window.ExampleExtension_fns.hello_world = hello_world;
		window.ExampleExtension_fns.hello_name = hello_name;
		window.ExampleExtension_fns.repeat_text = repeat_text;
		window.ExampleExtension_fns.is_even = is_even;
        `;
    document.body.appendChild(s);
})();