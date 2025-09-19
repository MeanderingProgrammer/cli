package main

import (
	"fmt"
	"log"
	"os"
	"path"
	"slices"
	"strings"

	"github.com/charmbracelet/huh"
	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/lipgloss/table"
)

const (
	Purple = lipgloss.Color("#9773f2")
	Gray   = lipgloss.Color("#c0c0c0")
	Cyan   = lipgloss.Color("#73daca")
	Green  = lipgloss.Color("#6bce69")
)

var (
	Border  = lipgloss.NewStyle().Foreground(Purple)
	Header  = lipgloss.NewStyle().Foreground(Purple).Padding(0, 1).Bold(true).Align(lipgloss.Center)
	Row     = lipgloss.NewStyle().Foreground(Gray).Padding(0, 1).MaxWidth(200)
	Skip    = lipgloss.NewStyle().Foreground(Cyan).Render
	Success = lipgloss.NewStyle().Foreground(Green).Render
)

type Alias struct {
	Name    string
	Command string
}

type Group struct {
	Name    string
	Aliases []Alias
}

func NewGroup(name string, aliases ...Alias) Group {
	return Group{Name: name, Aliases: aliases}
}

type Config struct {
	Groups []Group
}

func NewConfig() Config {
	groups := []Group{
		NewGroup(
			"general",
			Alias{Name: "reload", Command: "source ~/.zshrc"},
			Alias{Name: "update-sys", Command: "yadm pull && yadm bootstrap"},
			Alias{Name: "ll", Command: "ls -latrh --color=auto"},
			Alias{Name: "workspace", Command: "cd ~/dev/repos/personal"},
			Alias{Name: "notes", Command: "cd ~/Documents/notes"},
			Alias{Name: "wget", Command: "wget --hsts-file=${XDG_CACHE_HOME}/wget-hsts"},
		),
		NewGroup(
			"git",
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
		NewGroup(
			"tmux",
			Alias{Name: "tmux-exit", Command: "tmux kill-server"},
			Alias{Name: "tmux-switch", Command: "tmux switch-client -t \"$(tmux ls -F \"#S\" | fzf)\""},
		),
		NewGroup(
			"yadm",
			Alias{Name: "yb", Command: "~/.config/yadm/bootstrap"},
			Alias{Name: "ys", Command: "yadm status"},
			Alias{Name: "yl", Command: "yadm log"},
			Alias{Name: "yp", Command: "yadm push"},
			Alias{Name: "ypl", Command: "yadm pull"},
			Alias{Name: "ya", Command: "yadm add -u"},
			Alias{Name: "yc", Command: "yadm commit -m"},
			Alias{Name: "yac", Command: strings.Join([]string{
				"yadm add",
				"~/.config/alacritty/",
				"~/.config/bin/",
				"~/.config/fish/",
				"~/.config/ghostty/",
				"~/.config/git/",
				"~/.config/helix/",
				"~/.config/kitty/",
				"~/.config/lazygit/",
				"~/.config/mise/",
				"~/.config/npm/",
				"~/.config/nvim/",
				"~/.config/shellcheckrc",
				"~/.config/tmux/",
				"~/.config/vim/",
				"~/.config/wezterm/",
				"~/.config/yadm/",
				"~/.config/zsh/",
				"~/.ssh/config",
				"~/.ssh/config.d/00-git.conf",
				"~/docs/",
				"~/.zshenv",
				"~/.zshrc",
			}, " ")},
			Alias{Name: "yls", Command: "yadm list -a"},
			Alias{Name: "yd", Command: "yadm diff"},
			Alias{Name: "ydp", Command: "yadm diff HEAD"},
		),
		NewGroup(
			"pass",
			Alias{Name: "pas", Command: "pass git status"},
			Alias{Name: "pal", Command: "pass git log"},
			Alias{Name: "pap", Command: "pass git push"},
			Alias{Name: "papl", Command: "pass git pull"},
		),
		NewGroup(
			"advent",
			Alias{Name: "a-build", Command: "./scripts/advent.py build"},
			Alias{Name: "a-run", Command: "./scripts/advent.py run"},
			Alias{Name: "a-gen", Command: "./scripts/advent.py generate"},
			Alias{Name: "a-graph", Command: "./scripts/advent.py graph"},
		),
	}
	config := Config{groups}
	config.Validate()
	return config
}

func (c *Config) Validate() {
	groups, aliases := []string{}, []string{}
	for _, group := range c.Groups {
		if slices.Contains(groups, group.Name) {
			log.Fatal(fmt.Sprintf("Duplicate group: %s", group.Name))
		}
		groups = append(groups, group.Name)
		for _, alias := range group.Aliases {
			if slices.Contains(aliases, alias.Name) {
				log.Fatal(fmt.Sprintf("Duplicate alias: %s", alias.Name))
			}
			aliases = append(aliases, alias.Name)
		}
	}
}

func (c *Config) AliasMapping() ([]string, map[string]Alias) {
	keys, mapping := []string{}, make(map[string]Alias)
	for _, group := range c.Groups {
		for _, alias := range group.Aliases {
			key := fmt.Sprintf("%s: %s", alias.Name, alias.Command)
			keys = append(keys, key)
			mapping[key] = alias
		}
	}
	return keys, mapping
}

func (c *Config) GroupMapping() ([]string, map[string][]Alias) {
	keys, mapping := []string{}, make(map[string][]Alias)
	for _, group := range c.Groups {
		keys = append(keys, group.Name)
		mapping[group.Name] = group.Aliases
	}
	return keys, mapping
}

func main() {
	config := NewConfig()
	commands := map[string]func(Config){
		"Get by Alias":   getByAlias,
		"Get by Group":   getByGroup,
		"Update Aliases": updateAliases,
	}

	keys := []string{}
	for key := range commands {
		keys = append(keys, key)
	}
	slices.Sort(keys)
	command := getSelected("command", keys)

	commands[command](config)
}

func getByAlias(config Config) {
	keys, mapping := config.AliasMapping()
	selected := getMultiSelected("aliases", keys)
	aliases := []Alias{}
	for _, key := range selected {
		aliases = append(aliases, mapping[key])
	}
	renderAliases(aliases)
}

func getByGroup(config Config) {
	keys, mapping := config.GroupMapping()
	selected := getSelected("group", keys)
	aliases := mapping[selected]
	renderAliases(aliases)
}

func renderAliases(aliases []Alias) {
	t := table.New().
		Border(lipgloss.NormalBorder()).
		BorderStyle(Border).
		BorderRow(true).
		StyleFunc(func(row int, col int) lipgloss.Style {
			if row == 0 {
				return Header
			} else {
				return Row
			}
		}).
		Headers("Alias", "Command")
	for _, alias := range aliases {
		t.Row(alias.Name, alias.Command)
	}
	fmt.Println(t)
}

func updateAliases(config Config) {
	if !confirmAction("Are you sure you want to overwrite aliases?") {
		fmt.Println(Skip("Skipping update: user request"))
		return
	}

	lines := []string{}
	for _, group := range config.Groups {
		lines = append(lines, fmt.Sprintf("# %s", group.Name))
		for _, alias := range group.Aliases {
			lines = append(lines, fmt.Sprintf("alias %s='%s'", alias.Name, alias.Command))
		}
		lines = append(lines, "")
	}

	home := os.Getenv("HOME")
	aliasFilePath := path.Join(home, ".config/zsh/00-aliases.sh")
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
	fmt.Println(Success(fmt.Sprintf("Successfully updated: %s", aliasFilePath)))
}

func getSelected(name string, keys []string) string {
	var selected string
	err := huh.NewSelect[string]().
		Title(fmt.Sprintf("Select %s", name)).
		Options(huh.NewOptions(keys...)...).
		Value(&selected).
		Run()
	if err != nil {
		log.Fatal(err)
	}
	return selected
}

func getMultiSelected(name string, keys []string) []string {
	var selected []string
	err := huh.NewMultiSelect[string]().
		Title(fmt.Sprintf("Select %s (defaults to all if none selected)", name)).
		Options(huh.NewOptions(keys...)...).
		Value(&selected).
		Run()
	if err != nil {
		log.Fatal(err)
	}
	if len(selected) == 0 {
		return keys
	} else {
		return selected
	}
}

func confirmAction(title string) bool {
	var update bool
	err := huh.NewConfirm().
		Title(title).
		Value(&update).
		Run()
	if err != nil {
		log.Fatal(err)
	}
	return update
}
