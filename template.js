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
})();