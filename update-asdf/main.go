package main

import (
	"fmt"
	"log"
	"os/exec"
	"slices"
	"strings"

	"github.com/charmbracelet/huh"
	"github.com/charmbracelet/lipgloss"
)

const (
	Red    = lipgloss.Color("#db4b4b")
	Purple = lipgloss.Color("#9773f2")
	Orange = lipgloss.Color("#ff9e64")
	Cyan   = lipgloss.Color("#73daca")
	Green  = lipgloss.Color("#6bce69")
)

var (
	Error   = lipgloss.NewStyle().Bold(true).Foreground(Red).Render
	Title   = lipgloss.NewStyle().Foreground(Purple).Render
	Section = lipgloss.NewStyle().MarginLeft(2).Foreground(Orange).Render
	Skip    = lipgloss.NewStyle().MarginLeft(4).Foreground(Cyan).Render
	Action  = lipgloss.NewStyle().MarginLeft(4).Foreground(Green).Render
)

type Plugin struct {
	name    string
	current string
}

type Asdf struct {
	command   string
	unhandled []string
}

func NewAsdf() *Asdf {
	return &Asdf{
		command: "asdf",
		// Some plugins have complex versions so latest is not supported
		unhandled: []string{"java"},
	}
}

func (a *Asdf) Exists() bool {
	_, err := exec.LookPath(a.command)
	return err == nil
}

func (a *Asdf) Plugins() []*Plugin {
	// golang          1.22.0          ~/.tool-versions
	// java            temurin-21.0.1+12.0.LTS ~/.tool-versions
	current := a.runCommand("current")

	plugins := []*Plugin{}
	for _, details := range strings.Split(current, "\n") {
		fields := strings.Fields(details)
		plugin := &Plugin{
			name:    fields[0],
			current: fields[1],
		}
		if !slices.Contains(a.unhandled, plugin.name) {
			plugins = append(plugins, plugin)
		}
	}
	return plugins
}

func (a *Asdf) Latest(name string) string {
	// 1.22.0
	return a.runCommand("latest", name)
}

func (a *Asdf) Installed(name string) []string {
	// 3.11.7
	//   3.12.0
	//  *3.13.0
	installed := a.runCommand("list", name)

	versions := []string{}
	for _, version := range strings.Split(installed, "\n") {
		version = strings.TrimSpace(version)
		version = strings.TrimLeft(version, "*")
		versions = append(versions, version)
	}
	return versions
}

func (a *Asdf) Install(name string, version string) {
	a.runCommand("install", name, version)
}

func (a *Asdf) SetGlobal(name string, version string) {
	a.runCommand("global", name, version)
}

func (a *Asdf) Uninstall(name string, version string) {
	a.runCommand("uninstall", name, version)
}

func (a *Asdf) runCommand(arg ...string) string {
	out, err := exec.Command(a.command, arg...).CombinedOutput()
	if err != nil {
		log.Fatal(string(out), err)
	}
	return strings.TrimSpace(string(out))
}

func main() {
	asdf := NewAsdf()
	if !asdf.Exists() {
		fmt.Println(Error("asdf command does not exist"))
		return
	}

	plugins := asdf.Plugins()
	namePlugin := make(map[string]*Plugin)
	for _, plugin := range plugins {
		namePlugin[plugin.name] = plugin
	}
	for _, name := range getSelectedPlugins(plugins) {
		plugin := namePlugin[name]
		managePlugin(asdf, plugin)
	}
}

func getSelectedPlugins(plugins []*Plugin) []string {
	all, options := []string{}, []huh.Option[string]{}
	for _, plugin := range plugins {
		all = append(all, plugin.name)
		options = append(options, huh.NewOption(fmt.Sprintf("%s (%s)", plugin.name, plugin.current), plugin.name))
	}
	var selected []string
	form := huh.NewForm(
		huh.NewGroup(
			huh.NewMultiSelect[string]().
				Title("Select plugins to update (defaults to all if none selected)").
				Options(options...).
				Value(&selected),
		),
	)
	err := form.Run()
	if err != nil {
		log.Fatal(err)
	}
	if len(selected) == 0 {
		return all
	} else {
		return selected
	}
}

func managePlugin(asdf *Asdf, plugin *Plugin) {
	fmt.Println(Title(fmt.Sprintf("Handling: %s", plugin.name)))
	latest := asdf.Latest(plugin.name)
	plugin.current = updatePlugin(asdf, plugin, latest)
	for _, version := range asdf.Installed(plugin.name) {
		cleanupPlugin(asdf, plugin, version)
	}
	fmt.Println()
}

func updatePlugin(asdf *Asdf, plugin *Plugin, version string) string {
	fmt.Println(Section(fmt.Sprintf("Update: %s -> %s", plugin.current, version)))

	if plugin.current == version {
		fmt.Println(Skip("Skipped update: already using latest version"))
		return plugin.current
	}
	if !confirmAction() {
		fmt.Println(Skip("Skipped update: user request"))
		return plugin.current
	}

	asdf.Install(plugin.name, version)
	fmt.Println(Action(fmt.Sprintf("Installed version: %s", version)))
	asdf.SetGlobal(plugin.name, version)
	fmt.Println(Action(fmt.Sprintf("Set global version: %s", version)))
	return version
}

func cleanupPlugin(asdf *Asdf, plugin *Plugin, version string) {
	fmt.Println(Section(fmt.Sprintf("Cleanup: %s", version)))

	if plugin.current == version {
		fmt.Println(Skip("Skipped cleanup: currently in use"))
		return
	}
	if !confirmAction() {
		fmt.Println(Skip("Skipped cleanup: user request"))
		return
	}

	asdf.Uninstall(plugin.name, version)
	fmt.Println(Action(fmt.Sprintf("Uninstalled version: %s", version)))
}

func confirmAction() bool {
	var confirmed bool
	form := huh.NewForm(
		huh.NewGroup(
			huh.NewConfirm().
				Title("Confirm?").
				Value(&confirmed),
		),
	)
	err := form.Run()
	if err != nil {
		log.Fatal(err)
	}
	return confirmed
}
