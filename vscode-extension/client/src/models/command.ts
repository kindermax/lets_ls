export type CommandsMapping = {
    [key: string]: Command;
};

export class Command {
	constructor(public name: string, public description: string) {
		this.name = name;
		this.description = description;
	}
}