package main

import (
	"encoding/json"
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
	Section = lipgloss.NewStyle().Foreground(Orange).Render
	Skip    = lipgloss.NewStyle().Foreground(Cyan).Render
	Action  = lipgloss.NewStyle().Foreground(Green).Render
)

type Plugin struct {
	name    string
	version string
}

func NewPlugin(name, version string) Plugin {
	return Plugin{
		name:    name,
		version: version,
	}
}

func (p Plugin) Label() string {
	return fmt.Sprintf("%s@%s", p.name, p.version)
}

func (p Plugin) Compare(other Plugin) int {
	return strings.Compare(p.name, other.name)
}

type Install struct {
	Version string `json:"version"`
	Active  bool   `json:"active"`
}

type Mise struct {
	command string
}

func NewMise() (*Mise, error) {
	command := "mise"
	_, err := exec.LookPath(command)
	if err != nil {
		return nil, fmt.Errorf(Error("%s command does not exist"), command)
	}
	return &Mise{command: command}, nil
}

func (m *Mise) Active() ([]Plugin, error) {
	plugins := make(map[string][]Install)
	_, err := m.execute(&plugins, "ls", "--current", "--json")
	if err != nil {
		return nil, err
	}

	result := []Plugin{}
	for name, installs := range plugins {
		version := installs[0].Version
		result = append(result, NewPlugin(name, version))
	}
	slices.SortFunc(result, func(a, b Plugin) int {
		return a.Compare(b)
	})
	return result, nil
}

func (m *Mise) Inactive(name string) ([]Plugin, error) {
	installs := []Install{}
	_, err := m.execute(&installs, "ls", "--json", name)
	if err != nil {
		return nil, err
	}

	result := []Plugin{}
	for _, install := range installs {
		if !install.Active {
			result = append(result, NewPlugin(name, install.Version))
		}
	}
	return result, nil
}

func (m *Mise) Latest(name string) (Plugin, error) {
	output, err := m.execute(nil, "latest", name)
	version := strings.TrimSpace(string(output))
	return NewPlugin(name, version), err
}

func (m *Mise) Install(plugin Plugin) ([]byte, error) {
	log.Printf(Action("[installing] %s"), plugin.Label())
	return m.execute(nil, "install", plugin.Label())
}

func (m *Mise) SetGlobal(plugin Plugin) ([]byte, error) {
	log.Printf(Action("[setting global] %s"), plugin.Label())
	return m.execute(nil, "use", "--global", plugin.Label())
}

func (m *Mise) Uninstall(plugin Plugin) ([]byte, error) {
	log.Printf(Action("[uninstalling] %s"), plugin.Label())
	return m.execute(nil, "uninstall", plugin.Label())
}

func (m *Mise) execute(v any, arg ...string) ([]byte, error) {
	output, err := exec.Command(m.command, arg...).CombinedOutput()
	if err != nil {
		return nil, err
	}
	if v != nil {
		return nil, json.Unmarshal(output, v)
	} else {
		return output, nil
	}
}

func main() {
	mise, err := NewMise()
	if err != nil {
		log.Fatal(err)
	}

	plugins, err := mise.Active()
	if err != nil {
		log.Fatal(err)
	}

	chosen, err := choose(plugins)
	if err != nil {
		log.Fatal(err)
	}

	for _, plugin := range chosen {
		err = manage(mise, plugin)
		if err != nil {
			log.Fatal(err)
		}
	}
}

func choose(plugins []Plugin) ([]Plugin, error) {
	options := []huh.Option[string]{}
	for _, plugin := range plugins {
		options = append(options, huh.NewOption(plugin.Label(), plugin.name))
	}
	var selected []string
	err := huh.NewMultiSelect[string]().
		Title("Select plugins to update (all if none selected)").
		Options(options...).
		Value(&selected).
		Run()
	if err != nil {
		return nil, err
	}
	if len(selected) == 0 {
		return plugins, nil
	}
	result := []Plugin{}
	for _, plugin := range plugins {
		if slices.Contains(selected, plugin.name) {
			result = append(result, plugin)
		}
	}
	return result, nil
}

func manage(mise *Mise, current Plugin) error {
	log.Printf(Title("[manage] %s"), current.name)

	latest, err := mise.Latest(current.name)
	if err != nil {
		return err
	}

	err = update(mise, current, latest)
	if err != nil {
		return err
	}

	inactives, err := mise.Inactive(current.name)
	if err != nil {
		return err
	}
	if len(inactives) == 0 {
		log.Println(Section("[cleanup] no inactive"))
	}
	for _, inactive := range inactives {
		err = cleanup(mise, inactive)
		if err != nil {
			return err
		}
	}

	return nil
}

func update(mise *Mise, current Plugin, latest Plugin) error {
	log.Printf(Section("[update] %s -> %s"), current.version, latest.version)

	if current.version == latest.version {
		log.Println(Skip("[skipped] already using latest version"))
		return nil
	}

	perform, err := confirm()
	if err != nil {
		return err
	}
	if !perform {
		log.Println(Skip("[skipped] user request"))
		return nil
	}

	_, err = mise.Install(latest)
	if err != nil {
		return err
	}

	_, err = mise.SetGlobal(latest)
	return err
}

func cleanup(mise *Mise, plugin Plugin) error {
	log.Printf(Section("[cleanup] %s"), plugin.version)

	perform, err := confirm()
	if err != nil {
		return err
	}
	if !perform {
		log.Println(Skip("[skipped] user request"))
		return nil
	}

	_, err = mise.Uninstall(plugin)
	return err
}

func confirm() (bool, error) {
	var confirmed bool
	err := huh.NewConfirm().
		Title("Confirm?").
		Value(&confirmed).
		Run()
	return confirmed, err
}
