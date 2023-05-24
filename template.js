(function () {    
    class $EXTENSION_NAME extends Extension {
        constructor(ide) {
            super('$EXTENSION_NAME');
        }

        onOpenRole() {

        }

        getSettings() {
            return [
$SETTINGS
            ];
        }

        getMenu() {
            return {
$MENU
            };
        }

        getCategories() {
            return [
$CATEGORIES
            ];
        }

        getPalette() {
            return [
$PALETTE
            ];
        }

        getBlocks() {
            return [
$BLOCKS
            ];
        }

        getLabelParts() {
            return [
$LABELPARTS
            ];
        }

    }

    NetsBloxExtensions.register($EXTENSION_NAME);
    let path = document.currentScript.src;
    path = path.substring(0, path.lastIndexOf("/"));
    var s = document.createElement('script');
    s.type = "module";
    s.innerHTML = `import init, {$IMPORTS_LIST} from '${path}/pkg/netsblox_extension_rs.js';
    
    
        await init();

        window.$EXTENSION_NAME_fns = {};
$WINDOW_IMPORTS
        `;
    document.body.appendChild(s);
})();