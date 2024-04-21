package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"path"
	"slices"
	"strings"

	"github.com/charmbracelet/huh"
)

type Alias struct {
	Name    string
	Command string
}

type AliasGroup struct {
	Name    string
	Aliases []Alias
}

func (a *AliasGroup) ToMap() map[string]string {
	result := make(map[string]string)
	for _, alias := range a.Aliases {
		result[alias.Name] = alias.Command
	}
	return result
}

func NewAliasGroup(name string, aliases ...Alias) AliasGroup {
	return AliasGroup{Name: name, Aliases: aliases}
}

type AliasGroups struct {
	Groups []AliasGroup
}

func NewAliasGroups() AliasGroups {
	groups := []AliasGroup{
		NewAliasGroup(
			"General",
			Alias{Name: "reload", Command: "source ~/.zshrc"},
			Alias{Name: "update-sys", Command: "yadm pull && yadm bootstrap"},
			Alias{Name: "ll", Command: "ls -latr"},
			Alias{Name: "workspace", Command: "cd ~/dev/repos/personal"},
			Alias{Name: "notes", Command: "cd ~/Documents/notes"},
		),
		NewAliasGroup(
			"Git",
			Alias{Name: "gs", Command: "git status -uall"},
			Alias{Name: "gl", Command: "git log"},
			Alias{Name: "gp", Command: "git push"},
			Alias{Name: "gpl", Command: "git pull"},
			Alias{Name: "ga", Command: "git add --all"},
			Alias{Name: "gc", Command: "git commit -m"},
			Alias{Name: "gb", Command: "git branch"},
			Alias{Name: "gac", Command: "git add --all && git commit --amend"},
			Alias{Name: "gm", Command: "git checkout main"},
			Alias{Name: "gu", Command: "git branch -u main"},
			Alias{Name: "gr", Command: "git rebase -i"},
			Alias{Name: "gundo", Command: "git restore ."},
		),
		NewAliasGroup(
			"Yadm",
			Alias{Name: "yb", Command: "yadm bootstrap"},
			Alias{Name: "ys", Command: "yadm status"},
			Alias{Name: "yl", Command: "yadm log"},
			Alias{Name: "yp", Command: "yadm push"},
			Alias{Name: "ypl", Command: "yadm pull"},
			Alias{Name: "ya", Command: "yadm add -u"},
			Alias{Name: "yc", Command: "yadm commit -m"},
			Alias{Name: "yac", Command: strings.Join([]string{
				"yadm add",
				"~/docs/",
				"~/.config/alacritty/",
				"~/.config/git/",
				"~/.config/helix/",
				"~/.config/homebrew/",
				"~/.config/kitty/",
				"~/.config/lang/",
				"~/.config/npm/",
				"~/.config/nvim/",
				"~/.config/shell/",
				"~/.config/tmux/",
				"~/.config/wezterm/",
				"~/.config/yadm/",
			}, " ")},
			Alias{Name: "yls", Command: "yadm ls-files ~"},
			Alias{Name: "yd", Command: "yadm diff"},
		),
		NewAliasGroup(
			"Pass",
			Alias{Name: "pas", Command: "pass git status"},
			Alias{Name: "pal", Command: "pass git log"},
			Alias{Name: "pap", Command: "pass git push"},
			Alias{Name: "papl", Command: "pass git pull"},
		),
		NewAliasGroup(
			"Advent",
			Alias{Name: "a-build", Command: "./scripts/advent.py build"},
			Alias{Name: "a-run", Command: "./scripts/advent.py run"},
			Alias{Name: "a-gen", Command: "./scripts/advent.py generate"},
			Alias{Name: "a-graph", Command: "./scripts/advent.py graph"},
		),
	}
	result := AliasGroups{groups}
	result.GroupNames()
	result.AliasValues()
	return result
}

func (a *AliasGroups) GroupNames() []string {
	result := []string{}
	for _, group := range a.Groups {
		name := group.Name
		if slices.Contains(result, name) {
			log.Fatal(fmt.Sprintf("Found duplicate group name: %s", name))
		}
		result = append(result, name)
	}
	return result
}

func (a *AliasGroups) AliasValues() []string {
	result := []string{}
	for _, group := range a.Groups {
		for _, alias := range group.Aliases {
			name := alias.Name
			if slices.Contains(result, name) {
				log.Fatal(fmt.Sprintf("Found duplicate alias name: %s", name))
			}
			value := fmt.Sprintf("%s: %s", alias.Name, alias.Command)
			result = append(result, value)
		}
	}
	return result
}

func main() {
	aliasGroups := NewAliasGroups()
	commands := map[string]func(AliasGroups){
		"Get by Alias":   run_get_by_alias,
		"Get by Group":   run_get_by_group,
		"Update Aliases": run_update_aliases,
	}

	names := []string{}
	for name := range commands {
		names = append(names, name)
	}
	slices.Sort(names)
	name := getUserSelection("Select command", names)

	commands[name](aliasGroups)
}

func run_get_by_alias(aliasGroups AliasGroups) {
	selectedAliasValue := getUserSelection("Select alias", aliasGroups.AliasValues())
	fmt.Println(selectedAliasValue)
}

func run_get_by_group(aliasGroups AliasGroups) {
	selectedGroup := getUserSelection("Select group", aliasGroups.GroupNames())
	fmt.Println(selectedGroup)
	for _, group := range aliasGroups.Groups {
		if group.Name == selectedGroup {
			encoder := json.NewEncoder(os.Stdout)
			encoder.SetEscapeHTML(false)
			encoder.SetIndent("", "  ")
			encoder.Encode(group.ToMap())
		}
	}
}

func run_update_aliases(aliasGroups AliasGroups) {
	update := getUserUpdate()
	if !update {
		fmt.Println("Skipping update")
		return
	}

	lines := []string{}
	for _, group := range aliasGroups.Groups {
		lines = append(lines, fmt.Sprintf("# %s", group.Name))
		for _, alias := range group.Aliases {
			lines = append(lines, fmt.Sprintf("alias %s=\"%s\"", alias.Name, alias.Command))
		}
		lines = append(lines, "")
	}

	home := os.Getenv("HOME")
	aliasFilePath := path.Join(home, ".config/shell/aliases.sh")
	aliasFile, err := os.Create(aliasFilePath)
	if err != nil {
		log.Fatal(err)
	}
	defer aliasFile.Close()

	contents := strings.Join(lines, "\n")
	_, err = aliasFile.WriteString(contents)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Successfully updated %s\n", aliasFilePath)
}

func getUserSelection(title string, values []string) string {
	var selected string
	options := []huh.Option[string]{}
	for _, value := range values {
		option := huh.NewOption(value, value)
		options = append(options, option)
	}
	form := huh.NewForm(
		huh.NewGroup(
			huh.NewSelect[string]().
				Title(title).
				Options(options...).
				Value(&selected),
		),
	)
	err := form.Run()
	if err != nil {
		log.Fatal(err)
	}
	return selected
}

func getUserUpdate() bool {
	var update bool
	form := huh.NewForm(
		huh.NewGroup(
			huh.NewConfirm().
				Title("Are you sure you want to update aliases?").
				Value(&update),
		),
	)
	err := form.Run()
	if err != nil {
		log.Fatal(err)
	}
	return update
}
