package main

import (
	"fmt"
	"log"
	"os/exec"
	"slices"
	"strings"

	"github.com/charmbracelet/huh"
)

type Plugin struct {
	name    string
	current string
	latest  string
	updated bool
}

func NewPlugin(name string, current string) *Plugin {
	return &Plugin{
		name:    name,
		current: current,
		updated: false,
	}
}

func (p *Plugin) SetLatest(latest string) {
	p.latest = latest
}

func (p *Plugin) SetUpdated(updated bool) {
	p.updated = updated
}

type Asdf struct {
	command          string
	unhandledPlugins []string
}

func NewAsdf() *Asdf {
	return &Asdf{
		command: "asdf",
		// Some plugins have complex versions so latest is not supported
		unhandledPlugins: []string{"java"},
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
	plugins_details := strings.Split(current, "\n")

	plugins := []*Plugin{}
	for _, plugin_details := range plugins_details {
		fields := strings.Fields(plugin_details)
		plugin := NewPlugin(fields[0], fields[1])
		if !slices.Contains(a.unhandledPlugins, plugin.name) {
			plugins = append(plugins, plugin)
		}
	}
	return plugins
}

func (a *Asdf) Latest(name string) string {
	// 1.22.0
	return a.runCommand("latest", name)
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
		fmt.Println("asdf command does not exist")
		return
	}

	plugins := asdf.Plugins()

	namePlugin := make(map[string]*Plugin)
	for _, plugin := range plugins {
		namePlugin[plugin.name] = plugin
	}

	selectedNames := getUserSelectedNames(plugins)
	for _, name := range selectedNames {
		plugin := namePlugin[name]
		plugin.SetLatest(asdf.Latest(plugin.name))

		updated := updatePlugin(asdf, plugin)
		plugin.SetUpdated(updated)
	}
}

func updatePlugin(asdf *Asdf, plugin *Plugin) bool {
	fmt.Printf("Update %s: %s -> %s\n", plugin.name, plugin.current, plugin.latest)
	if plugin.current == plugin.latest {
		fmt.Println("  Skipping update: already using latest version")
		return false
	}

	upgrade := getUserConfirmation(fmt.Sprintf("Update %s: %s -> %s?", plugin.name, plugin.current, plugin.latest))
	if !upgrade {
		fmt.Println("  Skipping update: user request")
		return false
	}

	fmt.Printf("  Installing version %s\n", plugin.latest)
	asdf.Install(plugin.name, plugin.latest)
	fmt.Printf("  Setting global version %s\n", plugin.latest)
	asdf.SetGlobal(plugin.name, plugin.latest)

	cleanup := getUserConfirmation(fmt.Sprintf("Uninstall %s version %s?", plugin.name, plugin.current))
	if cleanup {
		fmt.Printf("  Uninstalling version %s\n", plugin.current)
		asdf.Uninstall(plugin.name, plugin.current)
	} else {
		fmt.Println("  Skipping cleanup: user request")
	}
	return true
}

func getUserSelectedNames(plugins []*Plugin) []string {
	var selectedNames []string

	allNames := []string{}
	options := []huh.Option[string]{}
	for _, plugin := range plugins {
		allNames = append(allNames, plugin.name)
		option := huh.NewOption(fmt.Sprintf("%s (%s)", plugin.name, plugin.current), plugin.name)
		options = append(options, option)
	}

	form := huh.NewForm(
		huh.NewGroup(
			huh.NewMultiSelect[string]().
				Title("Select plugins to update (defaults to all if none)").
				Options(options...).
				Value(&selectedNames),
		),
	)

	err := form.Run()
	if err != nil {
		log.Fatal(err)
	}
	if len(selectedNames) == 0 {
		return allNames
	} else {
		return selectedNames
	}
}

func getUserConfirmation(title string) bool {
	var confirmed bool

	form := huh.NewForm(
		huh.NewGroup(
			huh.NewConfirm().
				Title(title).
				Value(&confirmed),
		),
	)

	err := form.Run()
	if err != nil {
		log.Fatal(err)
	}
	return confirmed
}
