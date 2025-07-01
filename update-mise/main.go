package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
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

type State struct {
	Version   string `json:"version"`
	Active    bool   `json:"active"`
	Installed bool   `json:"installed"`
}

func NewState(version string, active bool, installed bool) State {
	return State{
		Version:   version,
		Active:    active,
		Installed: installed,
	}
}

type Plugin struct {
	Name  string
	State State
}

func NewPlugin(name string, state State) Plugin {
	return Plugin{
		Name:  name,
		State: state,
	}
}

func (p Plugin) Label() string {
	return fmt.Sprintf("%s@%s", p.Name, p.State.Version)
}

func (p Plugin) Compare(other Plugin) int {
	return strings.Compare(p.Name, other.Name)
}

type Mise struct {
	cmd string
}

func NewMise() (*Mise, error) {
	cmd := "mise"
	_, err := exec.LookPath(cmd)
	if err != nil {
		return nil, fmt.Errorf(Error("%s command does not exist"), cmd)
	}
	return &Mise{cmd: cmd}, nil
}

func (m *Mise) Current() ([]Plugin, error) {
	output, err := execute(m.cmd, []string{"ls", "-cJ"}, nil)
	if err != nil {
		return nil, err
	}

	plugins := make(map[string][]State)
	err = json.Unmarshal([]byte(output), &plugins)
	if err != nil {
		return nil, err
	}

	result := []Plugin{}
	for name, states := range plugins {
		state := states[0]
		result = append(result, NewPlugin(name, state))
	}
	slices.SortFunc(result, func(a, b Plugin) int {
		return a.Compare(b)
	})
	return result, nil
}

func (m *Mise) Inactive(name string) ([]Plugin, error) {
	output, err := execute(m.cmd, []string{"ls", "-J", name}, nil)
	if err != nil {
		return nil, err
	}

	states := []State{}
	err = json.Unmarshal([]byte(output), &states)
	if err != nil {
		return nil, err
	}

	result := []Plugin{}
	for _, state := range states {
		if !state.Active {
			result = append(result, NewPlugin(name, state))
		}
	}
	return result, nil
}

func (m *Mise) Latest(name string) (Plugin, error) {
	version, err := execute(m.cmd, []string{"latest", name}, nil)
	if err != nil {
		return Plugin{}, err
	}
	return NewPlugin(name, NewState(version, false, false)), nil
}

func (m *Mise) Install(plugin Plugin) (string, error) {
	log.Printf(Action("[installing] %s"), plugin.Label())
	env := []string{}
	if plugin.Name == "ruby" {
		openssl, err := execute("brew", []string{"--prefix", "openssl"}, nil)
		if err != nil {
			return "", err
		}
		env = append(env, "RUBY_CONFIGURE_OPTS=--with-openssl-dir="+openssl)
	}
	return execute(m.cmd, []string{"install", plugin.Label()}, env)
}

func (m *Mise) SetGlobal(plugin Plugin) (string, error) {
	log.Printf(Action("[setting global] %s"), plugin.Label())
	return execute(m.cmd, []string{"use", "--global", plugin.Label()}, nil)
}

func (m *Mise) Uninstall(plugin Plugin) (string, error) {
	log.Printf(Action("[uninstalling] %s"), plugin.Label())
	return execute(m.cmd, []string{"uninstall", plugin.Label()}, nil)
}

func execute(name string, arg []string, env []string) (string, error) {
	cmd := exec.Command(name, arg...)
	cmd.Env = append(os.Environ(), env...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(output)), nil
}

func main() {
	mise, err := NewMise()
	if err != nil {
		log.Fatal(err)
	}

	plugins, err := mise.Current()
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
		options = append(options, huh.NewOption(plugin.Label(), plugin.Name))
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
		if slices.Contains(selected, plugin.Name) {
			result = append(result, plugin)
		}
	}
	return result, nil
}

func manage(mise *Mise, current Plugin) error {
	log.Printf(Title("[manage] %s"), current.Name)

	latest, err := mise.Latest(current.Name)
	if err != nil {
		return err
	}

	err = update(mise, current, latest)
	if err != nil {
		return err
	}

	inactive, err := mise.Inactive(current.Name)
	if err != nil {
		return err
	}
	if len(inactive) == 0 {
		log.Println(Section("[cleanup] no inactive"))
	}
	for _, plugin := range inactive {
		err = cleanup(mise, plugin)
		if err != nil {
			return err
		}
	}

	return nil
}

func update(mise *Mise, current Plugin, latest Plugin) error {
	log.Printf(Section("[update] %s -> %s"), current.State.Version, latest.State.Version)

	if current.State.Installed && current.State.Version == latest.State.Version {
		log.Println(Skip("[skipped] already installed latest version"))
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
	log.Printf(Section("[cleanup] %s"), plugin.State.Version)

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
